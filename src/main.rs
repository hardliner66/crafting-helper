#[macro_use]
extern crate serde_derive;

use std::collections::BTreeMap;

use argparse::ArgumentParser;
use argparse::Collect;
use argparse::StoreFalse;
use argparse::StoreOption;
use argparse::StoreTrue;

macro_rules! s {
    ($e:expr) => {$e.to_owned()};
}

const MINUTES: u64 = 60;
const HOURS: u64 = MINUTES * 60;
const DAYS: u64 = HOURS * 24;
const WEEKS: u64 = DAYS * 7;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
enum MetaVar {
    StringVar(String),
    FloatVar(f64),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
struct CraftingTime {
    weeks: Option<u64>,
    days: Option<u64>,
    hours: Option<u64>,
    minutes: Option<u64>,
    seconds: Option<u64>,
}

impl CraftingTime {
    fn from_secs(secs: u64) -> CraftingTime {
        let mut secs = secs;
        let weeks = if secs > WEEKS {
            let result = secs / WEEKS;
            secs = secs - result * WEEKS;
            Some(result)
        } else {
            None
        };

        let days = if secs > DAYS {
            let result = secs / DAYS;
            secs = secs - result * DAYS;
            Some(result)
        } else {
            None
        };

        let hours = if secs > HOURS {
            let result = secs / HOURS;
            secs = secs - result * HOURS;
            Some(result)
        } else {
            None
        };

        let minutes = if secs > MINUTES {
            let result = secs / MINUTES;
            secs = secs - result * MINUTES;
            Some(result)
        } else {
            None
        };

        let seconds = if secs > 0 {
            Some(secs)
        } else {
            None
        };

        CraftingTime {
            seconds,
            minutes,
            hours,
            days,
            weeks,
        }
    }

    fn to_secs(&self) -> u64 {
        let mut time = self.weeks.unwrap_or(0) * WEEKS;
        time += self.days.unwrap_or(0) * DAYS;
        time += self.hours.unwrap_or(0) * HOURS;
        time += self.minutes.unwrap_or(0) * MINUTES;
        time += self.seconds.unwrap_or(0);
        time
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
struct Craftable {
    name: String,
    #[serde(default)]
    tier: i32,
    variations: Option<Vec<String>>,
    time: Option<CraftingTime>,
    requirements: Option<BTreeMap<String, f64>>,
    meta: Option<BTreeMap<String, MetaVar>>,
    count: Option<f64>,
}


#[derive(Debug)]
struct Options {
    name_or_id: String,
    path: Option<String>,
    details: bool,
    amount: Option<f64>,
    search: bool,
    ascending: bool,
    count: bool,
    list: bool,
}

fn get_options() -> Options {
    let mut name_or_id_parts: Vec<String> = vec![];
    let mut path = None;
    let mut details = false;
    let mut amount = None;
    let mut search = false;
    let mut ascending = true;
    let mut count = false;
    let mut list = false;
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Crafting helper.");
        ap.refer(&mut path)
            .add_option(&["-p", "--path"], StoreOption, "the path to the data file.");
        ap.refer(&mut details)
            .add_option(&["-d", "--details"], StoreTrue, "print details.");
        ap.refer(&mut amount)
            .add_option(&["-a", "--amount"], StoreOption, "amount needed.");
        ap.refer(&mut ascending)
            .add_option(&["-D", "--descending"], StoreFalse, "Sort descending.");
        ap.refer(&mut search)
            .add_option(&["-s", "--search"], StoreTrue, "search for all matching parts. prints info for part if only one is found.");
        ap.refer(&mut count)
            .add_option(&["-c", "--count"], StoreTrue, "show item count in data file");
        ap.refer(&mut list)
            .add_option(&["-l", "--list"], StoreTrue, "list all items in data file");
        ap.refer(&mut name_or_id_parts)
            .add_argument("search strings", Collect, "the name or id of the part you want to build. [partial names are supported (uses the first matching part), whitespaces are allowed and don't have to be escaped]").required();
        ap.parse_args_or_exit();
    }
    Options {
        name_or_id: name_or_id_parts.join(" "),
        path,
        details,
        amount,
        search,
        ascending,
        count,
        list,
    }
}

fn calculate(
    id: &String,
    data: &BTreeMap<String, Craftable>,
    parts: &mut BTreeMap<String, CraftingData>,
    depth: usize,
    amount: f64,
    print_tree: bool
) {
    let craftable = match &data.get(id) {
        Some(c) => c.clone(),
        None => {
            panic!(format!("Could not find key: {}", id));
        }
    };

    let amount = calc_count(amount, craftable.count);

    let prefix = if depth == 0 {
        s!("")
    } else {
        format!("- {0:<1$}", " ", (depth * 2) - 1)
    };

    let time = match &craftable.time {
        Some(t) => t.to_secs(),
        None => 0,
    };

    let count_string = format!("{}", amount);
    if count_string.contains(".") || count_string.contains(",") {
        println!("{}{}: {:.4} part(s) [Tier: {}]", prefix, craftable.name, amount, if craftable.tier > 0 { craftable.tier.to_string() } else { s!("Unknown") });
    } else {
        println!("{}{:<20}: {} part(s) [Tier: {}]", prefix, craftable.name, amount, if craftable.tier > 0 { craftable.tier.to_string() } else { s!("Unknown") });
    }
    println!();
    match parts.get_mut(&id.to_string()) {
        Some(part) => {
            part.amount += amount;
        }
        None => {
            parts.insert(id.clone(), CraftingData { amount, tier: craftable.tier, name: craftable.name.clone(), time_per_part: time });
        }
    }

    let part_count_div = craftable.count.unwrap_or(1.0);

    if let Some(requirements) = &craftable.requirements {
        for (name, part_count) in requirements {
            calculate(&name, &data, parts, depth + 1, (amount * (*part_count)) / part_count_div, print_tree);
        }
    }
}

struct CraftingData {
    name: String,
    tier: i32,
    amount: f64,
    time_per_part: u64,
}

const SEPARATOR: &str = "===========================================================";
const SEPARATOR_LONG: &str = "================================================================================";

fn indent_lines(value: String) -> String {
    value
        .split("\n")                                                    // split on newline
        .filter(|s| s.trim() != "")                                         // strip empty lines
        .map(|s| format!("{}- {}", " ".repeat(8), s.to_string().trim())) // indent each line
        .collect::<Vec<String>>().join("\n")                                // re-join lines
}

fn print_summary(name: &String, parts: &BTreeMap<String, CraftingData>, ascending: bool, print_details: bool) {
    let mut parts = parts.values().collect::<Vec<&CraftingData>>();
    parts.sort_by(|a, b| {
        let (a, b) = if !ascending {
            (b, a)
        } else {
            (a, b)
        };
        if a.tier.ne(&b.tier) {
            a.tier.cmp(&b.tier)
        } else {
            a.name.cmp(&b.name)
        }
    });

    let time = parts.iter().fold(0, |a, b| {
        let count = b.amount.ceil() as u64;
        a + (count * b.time_per_part)
    });

    if print_details {
        println!("{}", SEPARATOR);
        println!("{}", SEPARATOR);
        println!("========================= DETAILS =========================");
        println!("{}", SEPARATOR);
        println!("{}", SEPARATOR);
        let mut first = true;

        for part in parts {
            let count = part.amount.ceil() as u64;
            if count * part.time_per_part > 0 && !first {
                println!("{}", SEPARATOR);
            }
            first = true;
            let count_string = format!("{}", part.amount);
            if count_string.contains(".") || count_string.contains(",") {
                println!("{:.4} part(s)  {:<20}: [Tier: {}]", part.amount, part.name, if part.tier > 0 { part.tier.to_string() } else { s!("Unknown") });
            } else {
                println!("{:<6} part(s)  {:<20}: [Tier: {}]", part.amount, part.name, if part.tier > 0 { part.tier.to_string() } else { s!("Unknown") });
            }
            if count * part.time_per_part > 0 {
                println!("{:<7}total:", " ");
                println!("{:<8}",
                         indent_lines(
                         toml::to_string_pretty(&CraftingTime::from_secs(count * part.time_per_part)).unwrap()
                         )
                );

                println!("{:<7}per part:", " ");
                println!("{:<8}",
                         indent_lines(
                             toml::to_string_pretty(&CraftingTime::from_secs(part.time_per_part)).unwrap()
                         )
                );
            }
        }
    }

    println!();
    println!("{}", SEPARATOR_LONG);
    println!("{}", SEPARATOR_LONG);
    println!();

    println!("Total time for \"{}\"\n{}", name, toml::to_string_pretty(&CraftingTime::from_secs(time)).unwrap().trim());
}

fn find_matching(search_string: &String, data: &BTreeMap<String, Craftable>) -> Vec<(String, Craftable)> {
    let search_string = search_string.to_lowercase();

    let mut vec = Vec::new();

    for (id, craftable) in data.iter() {
        if id.to_lowercase() == search_string
            || id.to_lowercase().contains(&search_string)
            || craftable.name.to_lowercase() == search_string
            || craftable.name.to_lowercase().contains(&search_string) {
            vec.push((id.clone(), craftable.clone()));
        }
    }

    vec
}

fn calc_count(needed: f64, count: Option<f64>) -> f64 {
    if let Some(n) = count {
        (needed / n).ceil() * n
    } else {
        needed
    }
}

fn main() {
    let options = get_options();

    if options.name_or_id.trim() == "" && !options.list && !options.count {
        eprintln!("No search string given!");
        return;
    }

    let path = options.path.clone().unwrap_or(s!("data.toml"));
    let data = if std::path::Path::new(&path).exists() {
        toml::from_str::<BTreeMap<String, Craftable>>(&String::from_utf8(std::fs::read(&path).expect("Could not read file!")).expect("File not valid UTF-8!")).expect("Could not parse file!")
    } else {
        eprintln!("Could not find data file: \"{}\"", path);
        return;
    };

    if options.count {
        println!("Found {} items in datafile.", data.len());
        return;
    }

    if options.list {
        for (_, item) in &data {
            println!("{}", item.name);
        }
        return;
    }

    if options.search {
        let vec = find_matching(&options.name_or_id, &data);

        match vec.len() {
            0 => println!("no matching item found."),
            1 => {
                let (id, craftable) = &vec[0];
                println!("{}", id);
                println!();
                println!("{}", toml::to_string_pretty(craftable).unwrap())
            }
            _ => {
                for (id, craftable) in vec {
                    println!("{} => {}", id, craftable.name);
                }
            }
        }
    } else {
        let items = find_matching(&options.name_or_id, &data);
        match &items.len() {
            0 => println!("item not found."),
            1 => {
                let mut parts = BTreeMap::new();
                let (id, _) = &items[0];
                calculate(&id, &data, &mut parts, 0, options.amount.unwrap_or(1.0), !options.details);

                print_summary(&data[id].name, &parts, options.ascending, options.details);
            }
            _ => {
                println!("Found multiple elements matching input:");
                for (id, craftable) in &items {
                    println!("- [{}] \"{}\"", id, craftable.name);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_calc_count() {
        assert_eq!(calc_count(1.0, Some(4.0)), 4.0);
        assert_eq!(calc_count(4.0, Some(4.0)), 4.0);
        assert_eq!(calc_count(0.5, None), 0.5);
    }
}

