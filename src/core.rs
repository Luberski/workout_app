use chrono::{Local, Weekday};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::process::Command;
use std::slice::Iter;
use std::{fmt, fs, thread, time};

pub fn clear_terminal_screen() {
    if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/c", "cls"])
            .spawn()
            .expect("cls command failed to start")
            .wait()
            .expect("failed to wait");
    } else {
        Command::new("clear")
            .spawn()
            .expect("clear command failed to start")
            .wait()
            .expect("failed to wait");
    };
}

fn str_to_action(action: &str) -> MainAction {
    match action.trim() {
        "1" => MainAction::AddWorkout,
        "2" => MainAction::AddExercise,
        "0" => MainAction::Exit,
        _ => MainAction::Idle,
    }
}

struct Input {}

impl Input {
    pub fn new() -> Self {
        Self {}
    }

    fn get_input() -> io::Result<String> {
        let mut input: String = String::new();
        io::stdin().read_line(&mut input)?;
        input = input.trim().to_string();
        Ok(input)
    }

    fn yes_or_no() -> io::Result<bool> {
        match Input::get_input().expect("msg").trim() {
            "y" => return Ok(true),
            _ => return Ok(false),
        }
    }
}

#[derive(PartialEq, Debug)]
enum MainAction {
    AddWorkout,
    AddExercise,
    Exit,
    Idle,
}

impl fmt::Display for MainAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone)]
struct Profile {
    name: String,
    workout_days: Vec<Weekday>,
    workouts: Vec<Workout>,
}

impl Profile {
    fn new(name: String) -> Self {
        Self {
            name,
            workout_days: Vec::new(),
            workouts: Vec::new(),
        }
    }
}

pub struct System {
    curr_user: Profile,
    registered_users: Vec<Profile>,
    available_exercises: Vec<ExerciseType>,
}

impl System {
    pub fn new() -> std::io::Result<Self> {
        let mut profiles_list: Vec<Profile> = Vec::new();
        let mut profiles_handle = match File::open("profiles.txt") {
            Ok(file) => file,
            Err(_e) => match File::create("profiles.txt") {
                Ok(file) => file,
                Err(e) => return Err(e),
            },
        };

        let mut exercises_list: Vec<ExerciseType> = Vec::new();
        let mut exercises_handle = match File::open("exercise_list.txt") {
            Ok(file) => file,
            Err(_e) => match File::create("exercise_list.txt") {
                Ok(file) => file,
                Err(e) => return Err(e),
            },
        };

        // Populate
        let profiles_str = fs::read_to_string("profiles.txt")?;
        if profiles_str.is_empty() {
            for name in profiles_str.split(';') {
                profiles_list.push(Profile::new(name.to_string()));
            }
        }

        // let exercises_str = fs::read_to_string("exercise_list.txt")?;
        // if exercises_str.is_empty() {
        //     for name in exercises_str.split(';') {
        //         exercises_list.push(ExerciseType::new(name.to_string()));
        //     }
        // }

        Ok(Self {
            curr_user: Profile::new("None".to_string()),
            registered_users: profiles_list,
            available_exercises: exercises_list,
        })
    }

    pub fn login(&mut self) -> io::Result<()> {
        println!("Hello, please type your name to log in:");
        let mut buffer: String = String::new();
        io::stdin().read_line(&mut buffer)?;

        // Check if user exists in system
        match self.find_user(buffer.clone()) {
            Some(ref user) => self.curr_user = user.clone(),
            None => self.curr_user = self.create_new_user(String::from(buffer.trim()))?,
        }

        Ok(())
    }

    pub fn app(&mut self) -> io::Result<()> {
        println!("Welcome to the workout app, {}!", self.curr_user.name);

        let mut action: MainAction = MainAction::Idle;
        while action != MainAction::Exit {
            clear_terminal_screen();
            println!(" 1: Add workout\n 2: Add exercise\n 0: Exit");
            action = str_to_action(Input::get_input().unwrap().as_str());
            match action {
                MainAction::AddExercise => self.add_exercise(),
                MainAction::AddWorkout => self.add_workout(),
                _ => continue,
            }
        }

        Ok(())
    }

    fn find_user(&self, name: String) -> Option<Profile> {
        for user in self.registered_users.iter() {
            if user.name == name {
                return Some(user.clone());
            }
        }

        None
    }

    fn create_new_user(&mut self, name: String) -> std::io::Result<Profile> {
        let mut profiles_file_handle = match OpenOptions::new().write(true).open("profiles.txt") {
            Ok(handle) => handle,
            Err(e) => return Err(e),
        };
        let new_profile: Profile = Profile::new(name.clone());
        self.registered_users.push(new_profile.clone());
        profiles_file_handle.write((name + ";").as_bytes())?;

        Ok(new_profile)
    }

    pub fn add_workout(&mut self) {
        println!("Add workout");
        let mut exercises: Vec<Exercise> = Vec::new();
        let mut target_muscle: BodyPart = BodyPart::None;
        let mut exc_add_loop: bool = true;

        while exc_add_loop {
            println!("Choose target muscle");
            for muscle in BodyPart::iterator() {
                println!("{}", muscle.to_string());
            }
            match BodyPart::to_enum(&Input::get_input().unwrap()) {
                Some(muscle) => {
                    target_muscle = muscle;
                }
                None => continue,
            }

            while true {
                if (self.available_exercises.is_empty()) {
                    println!("There are no exercises available to use, create one? (y/n)");
                    if Input::yes_or_no().unwrap() {
                        self.add_exercise();
                    } else {
                        return;
                    }
                } // Check this if no exercises for given bodytype
                println!("Choose exercise:");
                let mut exercise: ExerciseType =
                    ExerciseType::new("none".to_string(), BodyPart::None);
                self.show_exercises_based_on_type(target_muscle.clone());
                match self.find_exercise(&Input::get_input().unwrap()) {
                    Some(item) => exercise = item,
                    None => continue,
                }

                println!("Choose amount of sets you made:");
                let workout_sets = WorkoutSet::new(Input::get_input().unwrap().parse().unwrap());
                println!("yo1");
                exercises.push(Exercise::new(exercise, workout_sets));
                println!("yo2");
                
                println!("Add another exercise? (y/n)");
                if !Input::yes_or_no().unwrap() {
                    exc_add_loop = false;
                    break;
                };
            }
        }

        self.curr_user.workouts.push(Workout::new(exercises));
    }

    pub fn add_exercise(&mut self) {
        let mut exc_loop: bool = true;

        while exc_loop {
            println!("Provide name for an exercise:");
            let exc_name = Input::get_input().unwrap();
            println!("Provide muscle target for this exercise:");
            for bodypart in BodyPart::iterator() {
                println!("{}", bodypart.to_string());
            }
            let exc_type = BodyPart::to_enum(Input::get_input().unwrap().as_str()).expect("wtf");
            self.available_exercises
                .push(ExerciseType::new(exc_name, exc_type));
            println!("Add another exercise?");
            if !Input::yes_or_no().unwrap() {
                exc_loop = false
            }
        }
    }

    fn find_exercise(&self, name: &str) -> Option<ExerciseType> {
        for exercise in self.available_exercises.iter() {
            if (exercise.name == name) {
                return Some(exercise.clone());
            }
        }

        None
    }

    fn show_exercises_based_on_type(&self, muscle: BodyPart) {
        for exercise in self.available_exercises.iter() {
            if (exercise.target == muscle) {
                println!("{}", exercise.name);
            }
        }
    }

    pub fn curr_user_exercises(&self) {
        for exercise in self.curr_user.workouts[0].exercises.iter() {
            exercise.details();
        }
    }
}

#[derive(Clone)]
struct Workout {
    date: chrono::DateTime<Local>,
    exercises: Vec<Exercise>,
}

impl Workout {
    fn new(exercises: Vec<Exercise>) -> Self {
        Self {
            date: Local::now(),
            exercises,
        }
    }
}

#[derive(Clone)]
struct ExerciseType {
    name: String,
    target: BodyPart,
}

impl ExerciseType {
    fn new(name: String, target: BodyPart) -> Self {
        Self { name, target }
    }
}

#[derive(Clone)]
struct Exercise {
    exc_type: ExerciseType,
    sets: WorkoutSet,
    notes: String,
}

impl Exercise {
    fn new(exc_type: ExerciseType, sets: WorkoutSet) -> Self {
        Self {
            exc_type,
            sets,
            notes: String::new(),
        }
    }

    fn details(&self) {
        println!(
            "{}, number of sets: {}, notes: {}",
            self.exc_type.name, self.sets.num_of_sets, self.notes
        );
    }
}

#[derive(Clone)]
struct WorkoutSet {
    num_of_sets: u8,
    reps: Vec<u8>,
}

impl WorkoutSet {
    fn new(num_of_sets: u8) -> Self {
        let mut reps: Vec<u8> = Vec::new();
        for set in 1..num_of_sets {
            println!("How many reps you got during {} set", set);
            reps.push(Input::get_input().unwrap().parse().unwrap());
        }

        Self { num_of_sets, reps }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum BodyPart {
    Chest = 0,
    Back = 1,
    Shoulders = 2,
    Triceps = 3,
    Biceps = 4,
    Abs = 5,
    Legs = 6,
    Calves = 7,
    None = 8,
}

impl fmt::Display for BodyPart {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl BodyPart {
    pub fn iterator() -> Iter<'static, BodyPart> {
        static BODYPARTS: [BodyPart; 8] = [
            BodyPart::Chest,
            BodyPart::Back,
            BodyPart::Shoulders,
            BodyPart::Triceps,
            BodyPart::Biceps,
            BodyPart::Abs,
            BodyPart::Legs,
            BodyPart::Calves,
        ];
        BODYPARTS.iter()
    }

    pub fn to_enum(str: &str) -> Option<Self> {
        match str.trim().to_lowercase().as_ref() {
            "chest" => Some(BodyPart::Chest),
            "back" => Some(BodyPart::Back),
            "shoulders" => Some(BodyPart::Shoulders),
            "triceps" => Some(BodyPart::Triceps),
            "biceps" => Some(BodyPart::Biceps),
            "abs" => Some(BodyPart::Abs),
            "legs" => Some(BodyPart::Legs),
            "calves" => Some(BodyPart::Calves),
            _ => None,
        }
    }
}
