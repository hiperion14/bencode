use std::ops::{Index, IndexMut};
use std::collections::HashMap;
use std::vec;

#[derive(Debug, Clone)]
pub enum BeeValue {
    Null,
    String(String),
    Raw(Vec<u8>),
    Integer(i128),
    List(Vec<Bee>),
    Dictionary(std::collections::HashMap<String, Bee>),
}


#[derive(Debug, Clone)]
pub struct Bee {
    bee_value: BeeValue,
    raw: Vec<u8>,
}

impl Index<&str> for Bee {
    type Output = Bee;

    fn index(&self, key: &str) -> &Bee {
        if let Bee {bee_value: BeeValue::Dictionary(map), raw: _} = self {
            map.get(key).unwrap()
        } else {
            panic!("Not compatible");
        }
    }
}

impl IndexMut<&str> for Bee {
    fn index_mut(&mut self, key: &str) -> &mut Self::Output {
        if let Bee {bee_value: BeeValue::Dictionary(map), raw: _} = self {
            map.entry(key.to_string()).or_insert(Bee { bee_value: BeeValue::Null, raw: vec![] })
        } else {
            panic!("Cannot index non-Object JSON value");
        }
    }
}

impl Index<usize> for Bee {
    type Output = Bee;

    fn index(&self, index: usize) -> &Self::Output {
        if let Bee {bee_value: BeeValue::List(vec), raw: _ } = self {
            vec.get(index).unwrap()
        } else {
            panic!("Not compatible");
        }
    }
}

impl IndexMut<usize> for Bee {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if let Bee {bee_value: BeeValue::List(vec), raw: _ } = self {
            if index >= vec.len() {
                vec.resize(index + 1, Bee {bee_value: BeeValue::Null, raw: vec![]});
            }
            &mut vec[index]
        } else {
            panic!("Cannot index non-Array JSON value");
        }
    }
}

impl Bee {
    pub fn get_string(&self) -> Option<String> {
        if let Bee { bee_value: BeeValue::String(s), raw: _ } = self {
            Some(s.to_string())
        } else {
            None
        }
    }

    pub fn get_int(&self) -> Option<i128> {
        if let Bee { bee_value: BeeValue::Integer(i), raw: _ } = self {
            Some(i.to_owned())
        } else {
            None
        }
    }
    
    pub fn get_raw(&self) -> Option<Vec<u8>> {
        if let Bee { bee_value: BeeValue::Raw(v), raw: _ } = self {
            Some(v.to_vec())
        } else {
            None
        }
    }

    pub fn get_decoded(&self) -> Vec<u8> {
        self.raw.to_vec()
    }
    
    pub fn get_list(&self) -> Option<Vec<Bee>> {
        if let Bee { bee_value: BeeValue::List(v), raw: _ } = self {
            Some(v.to_vec())
        } else {
            None
        }
    }

    pub fn get_dict(&self) -> Option<HashMap<String, Bee>> {
        if let Bee { bee_value: BeeValue::Dictionary(d), raw: _ } = self {
            Some(d.to_owned())
        } else {
            None
        }
    }
}

impl BeeValue {
    pub fn from_bytes(bytes: &Vec<u8>) -> Bee {
        let (value, _size) = Self::dfs(bytes, 0);
        return value;
    }

    fn dfs(bytes: &Vec<u8>, index: usize) -> (Bee, usize) {
        //String
        if bytes[index].is_ascii_digit() {
            let position = bytes.iter().skip(index).position(|&r| r == 0x3A).unwrap()+index;
            let length = String::from_utf8(bytes[index..position].to_vec()).unwrap().parse::<usize>().unwrap();
            let text = String::from_utf8(bytes[position+1..position+1+length].to_vec());
            if text.is_err() {
                return (Bee {
                    bee_value: BeeValue::Raw(bytes[position+1..position+1+length].to_vec()),
                    raw: bytes[index..position+1+length].to_vec() 
                }, position+1+length);
            }
            return (Bee {
                bee_value: BeeValue::String(text.unwrap()),
                raw: bytes[index..position+1+length].to_vec()
            }, position+1+length);
        }
        //Integer
        if bytes[index] == 0x69 {
            let position = bytes.iter().skip(index).position(|&r| r == 0x65).unwrap() + index;
            let number = String::from_utf8(bytes[index+1..position].to_vec()).unwrap().parse::<i128>().unwrap();
            return (Bee {
                bee_value: BeeValue::Integer(number),
                raw: bytes[index..position+1].to_vec()
            }, position+1);
        }
        //List
        if bytes[index] == 0x6C {
            let mut local_index = index+1;
            let mut list: Vec<Bee> = vec![];
            while bytes[local_index] != 0x65 {
                let value;
                (value, local_index) = Self::dfs(bytes, local_index);
                list.push(value);
            }
            return (Bee {
                bee_value: BeeValue::List(list),
                raw: bytes[index..local_index + 1].to_vec()
            }, local_index + 1);
        }
        //Dict
        if bytes[index] == 0x64 {
            let mut local_index = index+1;
            let mut dict = Bee { bee_value: BeeValue::Dictionary(HashMap::new()), raw: vec![]};
            let mut key: String = String::from("");
            let mut i = 0;
            while bytes[local_index] != 0x65 {

                let value;
                (value, local_index) = Self::dfs(bytes, local_index);

                if i % 2 == 1 {
                    dict[key.as_str()] = value;
                } else {
                    key = value.get_string().unwrap();
                }
                i+=1;
            }
            return (Bee {bee_value: dict.bee_value, raw: bytes[index..local_index + 1].to_vec()}, local_index + 1);
        }
        return (Bee {bee_value: BeeValue::Null, raw: vec![]}, index);
    }
}