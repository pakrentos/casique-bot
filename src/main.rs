use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};
use rand;

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone)]
pub enum State {
    Start{money: u32},
    ReceiveBet{money: u32},
}

impl Default for State {
    fn default() -> Self {
        Self::Start{money: 1000}
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting casique bot...");

    let bot = Bot.from_env().auto_send();

    Dispatcher::builder(
        bot,
        Update::filter_message()
            .enter_dialogue::<Message, InMemStorage<State>, State>()
            .branch(dptree::case![State::Start{money}].endpoint(start))
            .branch(dptree::case![State::ReceiveBet{money}].endpoint(receive_bet)),
    )
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .build()
        .setup_ctrlc_handler()
        .dispatch()
        .await;
}

async fn start(bot: AutoSend<Bot>, msg: Message, dialogue: MyDialogue, money: u32) -> HandlerResult {
    bot.send_message(msg.chat.id, "Hello, you have 1000 coins. Make a bet").await?;
    dialogue.update(State::ReceiveBet{money}).await?;
    Ok(())
}

async fn receive_bet(
    bot: AutoSend<Bot>,
    msg: Message,
    dialogue: MyDialogue,
    money: u32
) -> HandlerResult {
    let mut money = money;
    // let mut rng = rand::thread_rng();
    match msg.text().map(|text| text.parse::<u32>()) {
        Some(Ok(bet)) if bet > money => {
            bot.send_message(msg.chat.id, "You don't have enough money").await?;
        }
        Some(Ok(bet)) => {
            match (rand::random::<f32>() > 0.5) as u8 {
                0 => {
                    if bet > money {
                        bot.send_message(msg.chat.id, "You have lost everything!").await?;
                        dialogue.exit().await?;
                    }
                    else {
                        money -= bet;
                        let message = format!("You lost!{money} coins left");
                        bot.send_message(msg.chat.id, message).await?;
                        dialogue.update(State::ReceiveBet {money}).await?;
                    }
                }
                1 => {
                    money += bet;
                    let message = format!("You win! You have {money} coins now");
                    bot.send_message(msg.chat.id, message).await?;
                    dialogue.update(State::ReceiveBet {money}).await?;
                }
                _ => {bot.send_message(msg.chat.id, "Something went wrong, try again").await?;}
            }
        }
        _ => {bot.send_message(msg.chat.id, "Invalid input, just write me a single number").await?;}
    }
    Ok(())
}
