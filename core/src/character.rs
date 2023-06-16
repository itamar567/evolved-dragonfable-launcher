use std::str::FromStr;
use roxmltree::{Document, Node};
use serde::{Deserialize, Serialize, Serializer};
use serde::ser::SerializeStruct;
use crate::config::REMOTE_SERVER_URL;
use crate::REQWEST_CLIENT;

static mut CHARACTER: Option<Character> = None;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Item {
    id: String,
    name: String,
    item_amount: u32,
    max_item_amount: u32,
}

#[derive(Debug, Clone)]
struct Character {
    id: String,
    inventory: Vec<Item>,
    bank: Vec<Item>,

    current_quest_reward: Option<Item>,
}

impl Character {
    fn new(id: String) -> Self {
        Self {
            id,
            inventory: Vec::new(),
            bank: Vec::new(),
            current_quest_reward: None,
        }
    }
}

impl Serialize for Character {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut all_items = self.inventory.clone();
        all_items.extend(self.bank.clone().into_iter());

        let mut character = serializer.serialize_struct("Character", 2)?;
        character.serialize_field("id", &self.id)?;
        character.serialize_field("all_items", &all_items)?;

        character.end()
    }
}

pub async fn parse_post_request(data: &str, path: &str) {
    let character;
    unsafe {
        character = CHARACTER.as_mut();
    }

    match path.replace(REMOTE_SERVER_URL, "").as_str() {
        "/game/cf-characterload.asp" => parse_character_load(data),
        "/game/cf-bankload.asp" => character.unwrap().parse_bank_load(data),
        "/game/cf-questcomplete-Mar2011.asp" => character.unwrap().parse_quest_complete(data),
        "/game/cf-questreward.asp" => character.unwrap().parse_quest_reward(),
        _ => return,
    }

    unsafe {
        REQWEST_CLIENT.post("http://127.0.0.1:39621/character").json(&CHARACTER.clone()).send().await.unwrap();
    }
}

fn parse_item_node(item: Node) -> Option<Item> {
    if let Some(name) = item.attributes().find(|att| att.name() == "strItemName").map(|att| att.value()) {
        let amount = item.attributes().find(|att| att.name() == "intCount").map(|att| u32::from_str(att.value())).unwrap_or(Ok(1)).unwrap_or(1);
        if let Some(item_id) = item.attributes().find(|att| att.name() == "ItemID").map(|att| att.value()) {
            let max_item_amount;
            if let Some(Ok(item_amount)) = item.attributes().find(|att| att.name() == "intMaxStackSize").map(|att| u32::from_str(att.value())) {
                max_item_amount = item_amount;
            } else {
                max_item_amount = 1;
            }

            return Some(Item {
                id: item_id.to_string(),
                name: name.to_string(),
                item_amount: amount,
                max_item_amount,
            });
        }
    }

    None
}

fn parse_character_load(data: &str) {
    let doc = Document::parse(data).unwrap();
    let root = doc.root_element();

    let character_node = root.children().find(|node| node.tag_name().name() == "character").unwrap();
    let char_id = character_node.attributes().find(|att| att.name() == "CharID").unwrap().value();
    let items: Vec<Node> = character_node.children().filter(|node| node.tag_name().name() == "items").collect();

    let mut character = Character::new(char_id.to_string());

    for item in items {
        if let Some(item) = parse_item_node(item) {
            character.inventory.push(item);
        }
    }
    unsafe {
        CHARACTER = Some(character);
    }
}

impl Character {
    fn parse_bank_load(&mut self, data: &str) {
        let doc = Document::parse(data).unwrap();
        let root = doc.root_element();

        let bank_node = root.children().find(|node| node.tag_name().name() == "bank").unwrap();
        let items: Vec<Node> = bank_node.children().filter(|node| node.tag_name().name() == "items").collect();

        let mut bank_items = Vec::new();

        for item in items {
            if let Some(item) = parse_item_node(item) {
                bank_items.push(item);
            }
        }

        self.bank = bank_items;
    }

    fn parse_quest_reward(&mut self) {
        let item = self.current_quest_reward.take();
        if let Some(item) = item {
            if let Some(item_in_inventory) = self.inventory.iter_mut().find(|i| i.id == item.id) {
                if item_in_inventory.max_item_amount > item_in_inventory.item_amount {
                    item_in_inventory.item_amount += 1;
                }
                else {
                    self.inventory.push(item);
                }
            }
            else if let Some(item_in_bank) = self.bank.iter_mut().find(|i| i.id == item.id) {
                if item_in_bank.max_item_amount > item_in_bank.item_amount {
                    item_in_bank.item_amount += 1;
                }
                else {
                    self.bank.push(item);
                }
            }
            else {
                self.inventory.push(item);
            }
        }
    }

    fn parse_quest_complete(&mut self, data: &str) {
        let doc = Document::parse(data).unwrap();
        let quest_reward_node = doc.root_element().children().find(|node| node.tag_name().name() == "questreward").unwrap();
        let item_node = quest_reward_node.children().find(|node| node.tag_name().name() == "items").unwrap();

        self.current_quest_reward = parse_item_node(item_node);
    }
}