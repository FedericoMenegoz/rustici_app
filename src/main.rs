use std::{
    collections::HashMap,
    fs::{self, File},
    io,
    process::Command,
    time::{Duration, Instant},
};

use oxagaudiotool::{sound_config::OxAgSoundConfig, OxAgAudioTool};
use robotics_lib::{
    event::events::Event,
    world::{environmental_conditions::WeatherType, tile::TileType},
};

fn main() {
    println!("Rustici Project:\n- AI by Salvatore Cassarà\n- Visualizer 1 by Federico Menegoz\n- Visualizer 2 by Alberto Cimmino");
    println!("Please choose which visualizer to use (1-2):");
    let mut input = String::new();

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let trimmed = input.trim();

    match trimmed.parse::<u32>() {
        Ok(1) => {
            println!("Running visualizer_1...");
            // let status = Command::new("cargo")
            //     .args(&["run", "--release", "--bin", "visualizer_1"])
            //     .status()
            //     .expect("Failed to execute visualizer 1.");
            let status = Command::new("./target/release/visualizer_1")
            .status()
            .expect("Failed to execute visualizer 1.");
            // check for rewards.ron file if exists it is training time only if visualizer 1 exit correctly
            if status.success() {
                match File::open("rewards.ron") {
                    Ok(_) => {
                        let event_to_sound_config: HashMap<Event, OxAgSoundConfig> = HashMap::new();
                        let tile_type_to_sound_config: HashMap<TileType, OxAgSoundConfig> =
                            HashMap::new();
                        let weather_type_to_sound_config: HashMap<WeatherType, OxAgSoundConfig> =
                            HashMap::new();

                        let mut audio_tool = OxAgAudioTool::new(
                            event_to_sound_config,
                            tile_type_to_sound_config,
                            weather_type_to_sound_config,
                        )
                        .unwrap();
                        let _ = audio_tool
                            .play_audio(&OxAgSoundConfig::new("src/lib/audio/waiting_music.mp3"));

                        'a: for i in 0..1 {
                            println!("training: {i}");
                            let start = Instant::now();
                            // let mut training_session = Command::new("cargo")
                            //     .args(&["run", "--release", "--bin", "trainer"])
                            //     .spawn()
                            //     .expect("Error: could not run the trainer.");
                            let mut training_session = Command::new("./target/release/trainer")
                                // .args(&["run", "--release", "--bin", "trainer"])
                                .spawn()
                                .expect("Error: could not run the trainer.");

                            // sometime the world does not have enough content and the simulation does not end
                            // then we kill any simulation running more than 4 seconds
                            while start.elapsed() < Duration::new(4, 0) {
                                let mut terminated = false;
                                // if trainer process has terminated no need to wait 4 seconds just continue to next for iteration
                                training_session
                                    .try_wait()
                                    .map(|o| {
                                        if let Some(_) = o {
                                            terminated = true;
                                        }
                                    })
                                    .expect("Error checking child process.");
                                if terminated {
                                    continue 'a;
                                }
                            }

                            // kill the trainer
                            training_session.kill().expect("Error killing the trainer.");
                        }

                        // remove the rewards file
                        let _ = fs::remove_file("./rewards.ron");
                    }
                    Err(_) => {
                        println!("No user training detected. Terminating.")
                    }
                }
            }
        }
        Ok(2) => {
            println!("Running visualizer_2...");
            Command::new("./target/release/visualizer_2")
                .status()
                .expect("Error running visualizer 2.");
            // Command::new("cargo")
            //     .args(&["run", "--release", "--bin", "visualizer_2"])
            //     .status()
            //     .expect("Failed to execute visualizer 2.");
        }
        Ok(_) => panic!("Invalid input!"),
        Err(_) => panic!("Failed to parse input as number!"),
    }
}
