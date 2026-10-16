use core::{num, panic};
use std::{cmp, error::Error, fmt, io, process};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Department {
    name: String,
    location: String,
    room: String,
}

pub struct Res {
    department: Department,
    score: usize,
}

impl fmt::Display for Department {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\
            Department name: {}\n\
            Location: {}\n\
            Example Room: {}\n", 
         self.name, self.location, self.room)
    }
}

pub fn search(query: &str) {
    let data = match read_data() {
        Ok(departments) => departments,
        Err(e) => panic!("Error reading data: {e}"),
    };

    let mut substring_matches = Vec::new();
    let mut levenshtein_matches = Vec::new();

    for department in data {
        if department.name.contains(query){
            substring_matches.push(department);
        } else {
            let dist = levenshtein(&department.name, query);
            if dist < cmp::min(department.name.len(), query.len()) - 2 {
                levenshtein_matches.push(Res {department, score: dist}) 
            }
        }
    }

    levenshtein_matches.sort_by(|a, b| a.score.cmp(&b.score));

    println!("Substring matches:\n");
    for department in substring_matches {
        println!("{}", department);
    }
    println!("Misspelt matches:\n");
    if !levenshtein_matches.is_empty() {
        for i in 0..cmp::min(5, levenshtein_matches.len()) {
            println!("{}", levenshtein_matches[i].department);
        }
    }


}

pub fn add(name: String, location: String, room: String) {
    let mut departments = match read_data() {
        Ok(departments) => departments,
        Err(e) => panic!("Error reading data: {}", e),
    };

    departments.push( Department { name, location, room });
    write_data(departments).expect("Error writing data");
}

// might be replaced with damerau-levenshtein
pub fn levenshtein(s: &str, t: &str) -> usize {
    let s = s.as_bytes();
    let t = t.as_bytes();
    let n = s.len();
    let m = t.len();

    let mut row: Vec<usize> = (0..=m).collect();

    for i in 0..n {
        let mut previous_diagonal = row[0];
        row[0] = i + 1;

        for j in 0..m {
            let indicator = if s[i] == t[j] { 0 } else { 1 };

            let previous_above = row[j+1];

            row[j+1] = cmp::min(previous_above + 1, cmp::min(row[j] + 1, previous_diagonal + indicator));
            
            previous_diagonal = previous_above;
        }
    }

    row[m]
}

pub fn matches_substring(s: &str, t: &str) -> bool {
    t.contains(s)
}

fn read_data() -> Result<Vec<Department>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path("data.csv").expect("Error creating reader for csv");
    let mut result = Vec::new();

    for department in rdr.deserialize() {
        let department: Department = department?;
        result.push(department);
    }

    Ok(result)
}

fn write_data(departments: Vec<Department>) -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_path("data.csv").expect("Error creating writer for csv");

    for department in departments {
        match wtr.serialize(department) {
            Ok(_) => continue,
            Err(e) => panic!("Error writing data to data.txt: {}", e)
        };
    }

    Ok(())
}

