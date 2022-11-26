use std::{process, env, fs};

use chains::Predictor;

mod chains;


struct Config
{
    input_path: Option<String>,
    dictionary_path: Option<String>,
    save_path: Option<String>,
    amount: usize,
    text: Option<String>
}

impl Config
{
    pub fn parse(args: impl Iterator<Item=String>) -> Result<Self, String>
    {
        let mut dictionary_path = None;
        let mut input_path = None;
        let mut save_path = None;

        let mut amount = 10;

        let mut text = None;

        let mut args = args.peekable();
        while let Some(arg) = args.next()
        {
            if args.peek().is_none()
            {
                text = Some(arg);
                break;
            }

            match arg.as_str()
            {
                "-a" | "--amount" =>
                {
                    let value = args.next().ok_or_else(|| "-a must have a value".to_string())?;
                    amount = value.parse().map_err(|err| format!("cant parse {value}: {err}"))?;
                },
                "-d" | "--dictionary" =>
                {
                    let value = args.next().ok_or_else(|| "-d must have a value".to_string())?;
                    dictionary_path = Some(value);
                },
                "-i" | "--input" =>
                {
                    let value = args.next().ok_or_else(|| "-i must have a value".to_string())?;
                    input_path = Some(value);
                },
                "-s" | "--save" =>
                {
                    let value = args.next().ok_or_else(|| "-s must have a value".to_string())?;
                    save_path = Some(value);
                },
                x => return Err(x.to_string())
            }
        }

        if dictionary_path.is_none() && input_path.is_none()
        {
            return Err("-i or -d option are mandatory".to_string());
        }

        if save_path.is_none()
        {
            if let None = text
            {
                return Err("text not found".to_string());
            }
        }

        if save_path.is_some() && dictionary_path.is_some()
        {
            return Err("-s and -d cannot be used at the same time".to_string());
        }

        Ok(Config{dictionary_path, input_path, save_path, amount, text})
    }

    pub fn help_message() -> !
    {
        eprintln!("usage: {} [args] text", env::args().next().unwrap());
        eprintln!("args:");
        eprintln!("    -a, --amount        amount of words to predict (default 10)");
        eprintln!("    -d, --dictionary    path to a dictionary of words with probabilities");
        eprintln!("    -i, --input         path to a plaintext file with words");
        eprintln!("    -s, --save          if specified ignores text and saves dictionary into specified file");
        process::exit(1);
    }
}

fn main()
{
    let config = Config::parse(env::args().skip(1)).unwrap_or_else(|err|
    {
        eprintln!("invalid arguments: {err}");
        Config::help_message()
    });

    let predictor = if config.input_path.is_some() || config.save_path.is_some()
    {
        let path = config.input_path.unwrap_or_else(|| config.save_path.clone().unwrap());

        let file = fs::read_to_string(path.clone()).expect(&format!("cant read {path}"));
        let predictor = Predictor::create(Predictor::word_split(&file));

        if let Some(save_path) = config.save_path
        {
            predictor.save(&save_path).unwrap();
            return;
        }

        predictor
    } else
    {
        let path = config.dictionary_path.unwrap();
        Predictor::load(&path).expect(&format!("could load dictionary at {path}"))
    };

    let text = config.text.unwrap();
    let mut predicted = Predictor::word_split(&text)
        .map(String::from).collect::<Vec<String>>();

    for _ in 0..config.amount
    {
        let maybe_word = predictor.predict_word(&predicted
            .last().expect("text must not be empty"));

        if let Some(word) = maybe_word
        {
            predicted.push(word);
        } else
        {
            break;
        }
    }

    println!("{}", predicted.join(" "));
}