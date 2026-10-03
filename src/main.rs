use std::env;
use fluent_templates::{static_loader, Loader};
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
    utils::command::BotCommands,
};
use unic_langid::{langid, LanguageIdentifier};

static_loader! {
    static LOCALES = {
        locales: "./locales",
        fallback_language: "en",
    };
}

const DEFAULT_LANG: LanguageIdentifier = langid!("en");

fn get_locale(lang_code: Option<&str>) -> LanguageIdentifier {
    lang_code
        .and_then(|code| code.parse().ok())
        .unwrap_or_else(|| DEFAULT_LANG.clone())
}

fn tr(lang: &LanguageIdentifier, text_id: &str) -> String {
    LOCALES.lookup(lang, text_id)
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Commands:")]
enum Command {
    #[command(description = "show menu and banner.")]
    Start,
    #[command(description = "show help.")]
    Help,
}

fn menu_markup(lang: &LanguageIdentifier) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(tr(lang, "btn-bind-cur-unbinded"), "opt_1"),
            InlineKeyboardButton::callback(tr(lang, "btn-extract-now"), "opt_2"),
        ],
        vec![
            InlineKeyboardButton::callback(tr(lang, "btn-probe-db"), "opt_1"),
            InlineKeyboardButton::callback(tr(lang, "btn-take-screenshot"), "opt_2"),
        ],
        vec![
            InlineKeyboardButton::callback(tr(lang, "btn-manual-control"), "help"),
            InlineKeyboardButton::callback(tr(lang, "btn-logout"), "help"),
            InlineKeyboardButton::callback(tr(lang, "btn-clean-data"), "help")
        ],
    ])
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let tg_token = env::var("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN must be set");

    let bot = Bot::new(tg_token);

    let handler = dptree::entry()
        .branch(Update::filter_message().filter_command::<Command>().endpoint(cmd_handler))
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

async fn cmd_handler(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    let lang = get_locale(msg.from.as_ref().and_then(|u| u.language_code.as_deref()));

    match cmd {
        Command::Start => {
            bot.send_message(msg.chat.id, tr(&lang, "banner"))
                .reply_markup(menu_markup(&lang))
                .await?;
        }
        Command::Help => {
            bot.send_message(msg.chat.id, tr(&lang, "help-cmd"))
                .await?;
        }
    }
    Ok(())
}

async fn callback_handler(bot: Bot, q: CallbackQuery) -> ResponseResult<()> {
    let lang = get_locale(q.from.language_code.as_deref());

    if let Some(data) = &q.data {
        let text = match data.as_str() {
            "opt_1" => tr(&lang, "selected-opt-1"),
            "opt_2" => tr(&lang, "selected-opt-2"),
            "help" => tr(&lang, "help-text"),
            _ => tr(&lang, "unknown-opt"),
        };

        bot.answer_callback_query(q.id.clone()).text(&text).await?;

        if let Some(msg) = q.regular_message() {
            bot.send_message(msg.chat.id, text).await?;
        }
    }
    Ok(())
}
