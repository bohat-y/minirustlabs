use serde::Deserialize;

fn celsius_to_fahrenheit (celsius :f32) -> f32 {
    (celsius * (9.0/5.0)) + 32.0
}
#[derive(Debug)]
struct HygrometerReading {
    device_id : u32,
    humidity : f32,
    temperature : f32,
}
impl HygrometerReading {
    fn new (device_id :u32, humidity :f32, temperature :f32) -> Self {
        HygrometerReading {
            device_id,
            humidity,
            temperature,
        }
    }

    fn print (&self) {
        let fahrenheit = celsius_to_fahrenheit(self.temperature);

        println!("HygrometerReading reading:");
        println!("\tDevice ID:          {}", self.device_id);
        println!("\tHumidity:           {}%", self.humidity);
        println!("\tTemperature (°C):   {:.1}", self.temperature);
        println!("\tTemperature (°F):   {:.1}", fahrenheit);
        println!("-----------------------------");
    }
}
fn main() {


    let mut weather_hub : Vec<HygrometerReading> = Vec::new();

    weather_hub.push(HygrometerReading::new(1, 45.5, 23.0));
    weather_hub.push(HygrometerReading::new(1, 50.8, 15.0));
    weather_hub.push(HygrometerReading::new(2, 30.0, 30.0));
    weather_hub.push(HygrometerReading::new(2, 11.1, 40.0));

    println!("Weather Hub Readings \n");
    for h in &weather_hub {
        h.print();
    }


    let humidities :Vec<f32> = weather_hub.iter().map(| h |
        {let val =  h.humidity; dbg!(val); val}).collect();

    println!("Humidity values: {:?}\n", humidities);

    let device1_readings :Vec<&HygrometerReading> = weather_hub.iter().filter( | h | h.device_id == 1).collect();

    println!("Device 1 Readings \n");
    for device1_reading in &device1_readings {
        device1_reading.print();
    }


}



// A system is required that records data coming from a series of hygrometers.
//
// A hygrometer is a device that measures relative humidity and temperature in the air. Relative humidity, expressed as a percent, is a measure of the amount of water vapour that air is holding.
// A hygrometer also measures temperature in degrees Celsius.  Each hygrometer has a unique device ID.
//
// Write a function which converts Celsius to Fahrenheit.
//
// Design and develop a HygrometerReading structure. Add fields for the device ID (u32), humidity, and temperature. Add an implementation for the structure, provide a function print()
// which prints the details of the structure using println! including the temperature in both Celsius and Fahrenheit.
//
// A weather hub stores a collection of hygrometer readings. Create a vector to represent the weather hub, add 4 hygrometer readings (use device IDs 1 and 2).
//
// Loop over the weather hub calling print()
//
// Use map()to produce a new vector containing just the humidity readings values for the weather hub.
// ––
// Use filter() to produce a new vector containing just the hygrometer readings for a specific device id (e.g. 1)
//