use std::collections::HashMap;
use std::fmt::Display;

pub type Result<T> = core::result::Result<T, anyhow::Error>;

#[derive(Debug, Clone)]
pub struct Kuma {
    pub url: String,
    pub auth: String,
}

impl Kuma {
    pub fn new(url: impl Into<String>, auth: impl Into<String>) -> Self {
        let mut url = url.into();

        if url.contains("https://") {
            (0..8).for_each(|_| {
                url.remove(0);
            });
        };

        Self {
            url,
            auth: format!("https://:{}", auth.into()),
        }
    }

    // infallible: returns offline if server cannot be reached
    pub async fn get_abstract(&self) -> DataHouse {
        if let Ok(data) = self.get().await {
            return data;
        };
        DataHouse {
            entries: HashMap::new(),
            state: HouseState::Offline,
        }
    }
    // gets the status based on authentication and uri
    pub async fn get(&self) -> Result<DataHouse> {
        let uri = format!("{}@{}", self.auth, self.url);
        println!("URI: {uri}");
        let data: Vec<Data> = reqwest::get(uri)
            .await?
            .text()
            .await?
            .split('\n')
            .filter(|a| a.contains("monitor_status{monitor_name="))
            .map(|a| {
                let mut a: Vec<String> = a.split("").map(String::from).collect();
                for _ in 0..15 {
                    a.remove(0);
                }
                a.into_iter().fold(String::new(), |mut a, b| {
                    a += &b;
                    a
                })
            })
            .map(|a| {
                let mut a: Vec<String> = a
                    .replace(['"', '\\'], "")
                    .split(',')
                    .filter_map(|a| a.split('=').map(String::from).last())
                    .enumerate()
                    .map(|a| {
                        if a.0 != 4 {
                            return a.1;
                        }
                        a.1.split("")
                            .filter_map(|a| a.parse::<u32>().ok())
                            .sum::<u32>()
                            .to_string()
                    })
                    .collect();
                a.remove(3);
                a
            })
            .filter_map(|a| {
                Some(Data {
                    monitor_name: a.first()?.clone(),
                    monitor_type: MonitorType::from_str(a.get(1)?),
                    monitor_url: a.get(2)?.clone(),
                    status: Status::from_str(a.get(3)?)?,
                })
            })
            .collect();

        Ok(DataHouse::from_vec(data))
    }
}
#[derive(Debug, Clone, Ord, PartialOrd, PartialEq, Eq)]
pub struct Data {
    pub monitor_name: String,
    pub monitor_type: MonitorType,
    pub monitor_url: String,
    pub status: Status,
}

#[derive(Debug, Clone)]
pub struct DataHouse {
    pub entries: HashMap<String, Data>,
    pub state: HouseState,
}
impl DataHouse {
    pub fn offline_services(&self) -> HashMap<String, Data> {
        self.clone()
            .entries
            .into_iter()
            .filter(|a| a.1.status == Status::Offline)
            .collect()
    }
    fn from_vec(input: Vec<Data>) -> Self {
        let mut map = HashMap::new();
        for x in input {
            map.insert(x.monitor_name.clone(), x);
        }

        let (offline_count, online_count) = map.values().fold((0, 0), |(mut of, mut on), b| {
            match b.status {
                Status::Online => on += 1,
                Status::Offline => of += 1,
            };
            (of, on)
        });
        Self {
            entries: map,
            state: match (offline_count, online_count) {
                (0, 0) => HouseState::Offline,
                (0, _) => HouseState::Online,
                (_, 0) => HouseState::Offline,
                (a, _) => HouseState::Degraded(a),
            },
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum HouseState {
    // No services can be reached, fully offline
    Offline,
    // Partially offline, issues with certain services
    Degraded(usize),
    // fully online
    Online, // Something went wrong with collecting data
}
impl Display for HouseState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            HouseState::Offline => "Offline",
            HouseState::Degraded(_) => "Degraded",
            HouseState::Online => "Online",
        }
        .to_string();
        write!(f, "{}", str)
    }
}
impl HouseState {
    pub fn is_degraded(&self) -> Option<usize> {
        match self {
            HouseState::Offline | HouseState::Online => None,
            HouseState::Degraded(a) => Some(*a),
        }
    }
}
#[derive(Debug, Clone, Ord, PartialOrd, PartialEq, Eq)]
pub enum MonitorType {
    Http,
    Other,
}
impl Display for MonitorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            MonitorType::Http => "HTTP/HTTPS",
            MonitorType::Other => "Other",
        }
        .to_string();
        write!(f, "{}", str)
    }
}
impl MonitorType {
    fn from_str(i: impl Into<String>) -> Self {
        match i.into().as_str() {
            "http" => MonitorType::Http,
            _ => MonitorType::Other,
        }
    }
}
#[derive(Debug, Clone, Ord, PartialOrd, PartialEq, Eq)]
pub enum Status {
    Online,
    Offline,
}
impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Status::Online => "Online",
            Status::Offline => "Offline",
        }
        .to_string();
        write!(f, "{}", str)
    }
}
impl Status {
    fn from_str(i: impl Into<String>) -> Option<Self> {
        match i.into().parse::<u32>().ok()? {
            0 => Some(Status::Offline),
            1 => Some(Status::Online),
            _ => None,
        }
    }
}
