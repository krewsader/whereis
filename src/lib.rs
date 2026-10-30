use std::{
    cmp,
    error::Error,
    fmt::{self, Write},
    time::Instant,
};

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
        writeln!(f, "<strong>Department name:</strong> {}", self.name)?;
        writeln!(f, "<strong>Location:</strong> {}", self.location)?;
        writeln!(f, "<strong>Example Room:</strong> {}", self.room)?;
        writeln!(
            f,
            "<a href=\"/modify?name={}\"></strong>Modify</a> <a href=\"/delete?name={}\"></strong>Delete</a>",
            self.name, self.name
        )
    }
}

pub fn search(query: &str) -> String {
    let now = Instant::now();

    let data = match read_data() {
        Ok(departments) => departments,
        Err(_) => {
            return String::from("Error reading data");
        }
    };

    let mut substring_matches = Vec::new();
    let mut levenshtein_matches = Vec::new();

    for department in data {
        if department
            .name
            .to_lowercase()
            .contains(&query.to_lowercase())
        {
            substring_matches.push(department);
        } else {
            let dist = levenshtein(&department.name, query);
            if dist < cmp::max(department.name.len(), query.len()) - 2 {
                levenshtein_matches.push(Res {
                    department,
                    score: dist,
                })
            }
        }
    }

    levenshtein_matches.sort_by(|a, b| a.score.cmp(&b.score));
    let num_results = substring_matches.len() + cmp::min(5, levenshtein_matches.len());
    let elapsed = now.elapsed();
    let mut output = String::new();

    writeln!(
        &mut output,
        "<p style=\"margin: 5px\">Found {} results in {} ns ({} ms)</p>",
        num_results,
        elapsed.as_nanos(),
        elapsed.as_millis()
    )
    .unwrap_or_else(|_| ());
    writeln!(
        &mut output,
        "<div class=\"results\"><h2>Substring matches:</h2>"
    )
    .unwrap_or_else(|_| ());
    for department in substring_matches {
        writeln!(&mut output, "<div class=\"result\">{}</div>", department).unwrap_or_else(|_| ());
    }
    writeln!(
        &mut output,
        "</div><div class=\"results\"><h2>Misspelt matches:</h2>"
    )
    .unwrap_or_else(|_| ());
    if !levenshtein_matches.is_empty() {
        for i in 0..cmp::min(5, levenshtein_matches.len()) {
            writeln!(
                &mut output,
                "<div class=\"result\">{}<div>",
                levenshtein_matches[i].department
            )
            .unwrap_or_else(|_| ());
        }
    }
    writeln!(&mut output, "</div>").unwrap_or_else(|_| ());

    output
}

pub fn add(name: String, location: String, room: String) -> Result<(), &'static str> {
    let mut departments = match read_data() {
        Ok(departments) => departments,
        Err(_) => return Err("Error reading data"),
    };

    let s = name.to_lowercase();

    for department in &departments {
        let t = department.name.to_lowercase();
        if t.contains(&s) || s.contains(&t) {
            println!("{} {}", s, t);
            return Err("Department already exists in database");
        }
    }

    departments.push(Department {
        name,
        location,
        room,
    });
    match write_data(departments) {
        Ok(_) => {}
        Err(_) => {
            return Err("Error writing data");
        }
    }
    Ok(())
}

pub fn modify(name: String, location: String, room: String) -> Result<(), &'static str> {
    let mut departments = match read_data() {
        Ok(departments) => departments,
        Err(_) => return Err("Error reading data"),
    };
    for department in &mut departments {
        if department.name == name {
            if location != "" {
                department.location = location;
            }
            department.room = room;
            break;
        }
    }

    match write_data(departments) {
        Ok(_) => {}
        Err(_) => {
            return Err("Error writing data");
        }
    }

    Ok(())
}

pub fn delete(name: String) -> Result<(), &'static str> {
    let departments = match read_data() {
        Ok(departments) => departments,
        Err(_) => return Err("Error reading data"),
    };

    let mut result = Vec::new();
    let mut found = false;

    for department in departments {
        if department.name != name {
            result.push(department);
        } else {
            found = true;
        }
    }

    if !found {
        return Err("Department not found");
    }

    match write_data(result) {
        Ok(_) => {}
        Err(_) => return Err("Error writing data"),
    }

    Ok(())
}

// might be replaced with damerau-levenshtein
pub fn levenshtein(s: &str, t: &str) -> usize {
    let binding = s.to_lowercase();
    let s = binding.as_bytes();
    let binding = t.to_lowercase();
    let t = binding.as_bytes();
    let n = s.len();
    let m = t.len();

    let mut row: Vec<usize> = (0..=m).collect();

    for i in 0..n {
        let mut previous_diagonal = row[0];
        row[0] = i + 1;

        for j in 0..m {
            let indicator = if s[i] == t[j] { 0 } else { 1 };

            let previous_above = row[j + 1];

            row[j + 1] = cmp::min(
                previous_above + 1,
                cmp::min(row[j] + 1, previous_diagonal + indicator),
            );

            previous_diagonal = previous_above;
        }
    }

    row[m]
}

pub fn levenshtein_damerau(s: &str, t: &str) -> usize {
    let s = s.as_bytes();
    let t = t.as_bytes();
    let n = s.len();
    let m = t.len();

    let mut dp: Vec<Vec<usize>> = Vec::new();

    dp.push((0..=m).collect());
    for j in 0..=m {
        dp[j].push(j);
    }

    0
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
            Err(e) => return Err(Box::new(e)),
        };
    }

    Ok(())
}
