fn main() {
    let cargos: Vec<Dependency> = vec![
        Dependency {
            name: String::from("factorial"),
            version: Version { major: 1, minor: 3, patch: 0 },
            cvss: 0.0,
        },
        Dependency {
            name: String::from("rand"),
            version: Version { major: 1, minor: 8, patch: 5 },
            cvss: 3.0,
        },
        Dependency {
            name: String::from("libc"),
            version: Version { major: 1, minor: 2, patch: 139 },
            cvss: 8.9,
        },
    ];

    println!("=============================\n");

    for c in &cargos {
        println!("{:}", c.info());
    }

    println!("=============================\n");

    let no_vuls: Vec<&Dependency> = cargos.iter().filter( | c | c.cvss == 0.0).collect();
    if no_vuls.is_empty() {
        println!("No safe cargos found.");
    } else {
        println!("Cargos with no vulnerabilities:");
        for c in &no_vuls {
            println!("{}", c.info());
        }
    }

    println!("=============================\n");

    let total :f32 = cargos.iter().fold(0.0, |acc, c| acc + c.cvss);
    println!("Total CVSS score: {:.2}", total);

    println!("=============================\n");


}

#[derive(Debug)]
struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

#[derive(Debug)]
struct Dependency{
    name: String,
    version: Version,
    cvss: f32
}

impl Version {
    fn info(&self) -> String{
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Dependency {
    // e.g. rand = “1.8.5”, CVSS 3.0 Low
    fn info(&self) -> String {
        format!("{} = \"{}\", CVSS {:.1} {}", self.name, self.version.info(), self.cvss, self.severity())
    }

    fn severity(&self) -> &str{
        // rust doesnt support float ranges in match arms

        if self.cvss == 0.0{
            "None"
        } else if self.cvss <= 3.9{
            "Low"
        } else if self.cvss <= 6.9 {
            "Medium"
        } else if self.cvss <= 8.9 {
            "High"
        } else {
            "Critical"
        }
    }
}
