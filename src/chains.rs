use std::{
    fs::File,
    io,
    collections::HashSet
};

use serde::{Deserialize, Serialize};

use rand::{self, Rng};


#[derive(Debug, Deserialize, Serialize)]
struct DictionaryWord
{
    name: String,
    total: u32,
    amounts: Vec<u32>
}

struct SortedDictionary<'a>
{
    words: Vec<&'a str>
}

impl<'a> SortedDictionary<'a>
{
    pub fn new(mut words: Vec<&'a str>) -> Self
    {
        words.sort_unstable();
        SortedDictionary{words}
    }

    pub fn words(&self) -> Vec<&'a str>
    {
        self.words.clone()
    }

    pub fn len(&self) -> usize
    {
        self.words.len()
    }

    pub fn index_of<'b>(&self, other: &'b str) -> Option<usize>
    {
        self.words.binary_search(&other).ok()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Predictor
{
    words: Vec<DictionaryWord>
}

impl Predictor
{
    pub fn word_split<'a>(text: &'a String) -> impl Iterator<Item=&'a str> + Clone
    {
        text.split(&[' ', '\n', ',', '.']).filter(|v| v.len()!=0)
    }

    pub fn create<'a>(words: impl Iterator<Item=&'a str> + Clone) -> Self
    {
        let mut unique_words = HashSet::new();

        for word in words.clone()
        {
            if !unique_words.contains(&word)
            {
                unique_words.insert(word);
            }
        }

        let dictionary = SortedDictionary::new(unique_words.into_iter()
            .collect::<Vec<&'a str>>());

        let words_len = dictionary.len();
        let mut out_words = dictionary.words().into_iter().map(|word|
            {
                DictionaryWord{
                    name: word.to_owned(),
                    total: 0,
                    amounts: vec![0; words_len]
                }
            }).collect::<Vec<DictionaryWord>>();

        let mut words_iter = words.into_iter().peekable();
        while let Some(word) = words_iter.next()
        {
            if words_iter.peek().is_some()
            {
                let current_index = dictionary.index_of(word).unwrap();
                let next_index = dictionary.index_of(words_iter.peek().unwrap()).unwrap();

                out_words[current_index].total += 1;
                out_words[current_index].amounts[next_index] += 1;
            }
        }

        Predictor{words: out_words}
    }

    fn index_of(&self, other: &str) -> Option<usize>
    {
        self.words.binary_search_by_key(&other, |dictionary_word| &dictionary_word.name).ok()
    }

    pub fn predict_word(&self, word: &str) -> Option<String>
    {
        if let Some(index) = self.index_of(word)
        {
            let mut rng = rand::thread_rng();

            let current_word = &self.words[index];

            if current_word.total==0
            {
                return None;
            }

            let picked_amount = rng.gen_range(0..current_word.total);

            let mut counter = 0;
            for (i, amount) in current_word.amounts.iter().enumerate()
            {
                counter += amount;

                if counter > picked_amount
                {
                    return Some(self.words[i].name.clone())
                }
            }

            return None;
        } else
        {
            None
        }
    }

    pub fn load(filename: &str) -> Result<Self, ciborium::de::Error<io::Error>>
    {
        ciborium::de::from_reader::<Self, _>(File::open(filename)
            .map_err(|err| ciborium::de::Error::Io(err))?)
    }

    pub fn save(&self, filename: &str) -> Result<(), ciborium::ser::Error<io::Error>>
    {
        ciborium::ser::into_writer(&self, File::create(filename)
            .map_err(|err| ciborium::ser::Error::Io(err))?)
    }
}