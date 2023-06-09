pub mod get;
pub mod post;
mod stream;
mod utils;

use reqwest::Client;
use std::sync::LazyLock;

static REQWEST_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);
