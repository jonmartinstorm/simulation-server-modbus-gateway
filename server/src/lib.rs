pub mod utils {
    
    pub mod watertank {
        use serde::{Serialize, Deserialize};
        use rand::prelude::*;
        use rand_distr::{Normal, Distribution};

        // How many cubic MM in one liter, Thousand liters in one cubic M. 
        const L_TO_CUBIC_MM: f32 = 1000000.0;
        /// A water tank struct
        /// There is flow into the water tank a size of the water tank and a level we want the water to be
        /// There is an areal of the water tank, it is a box watertank with a hight
        /// There is also a valve out of the water tank that controls the outflow
        /// 
        #[derive(Serialize, Deserialize, Debug, Copy, Clone)]
        pub struct WaterTank {
            pub level: f32,         // the water level of the tank mm. 
            pub inflow_mean: f32,   // the mean inflow if the tank l/s
            pub inflow_stddev: f32, // the stddev of inflow of the tank l/s
            pub inflow: f32,        // the inflow right now
            pub areal: f32,         // the areal of the tank mm^2
            pub height: f32,        // the height of the tank mm
            pub outflow: f32,       // the outflow of the tank l/s
            pub max_outflow: f32,
            pub set_level: f32,     // the wanted level of the tank mm, Real value? or 4 - 20 mA?
        }
    
        impl WaterTank {
            fn _volume(&self) -> f32 {
                self.areal * self.height
            }
    
            pub fn update_level(&mut self, seconds_passed: f32) {
                // water volume of the tank = areal * level
                // change in volume = volume + (inflow - outflow) * seconds_passed
                let volume = (self.areal * self.level) + ((self.inflow - self.outflow) * seconds_passed * L_TO_CUBIC_MM );
                self.level = volume / self.areal;
            }
    
            pub fn update_inflow(&mut self) {
                let mut rng = thread_rng();
                let normal = Normal::new(self.inflow_mean, self.inflow_stddev).unwrap();
                let v = normal.sample(&mut rng);
                self.inflow = v;
            }
        }
    }

    pub mod protocol {
        use serde::{Serialize, Deserialize};

        #[derive(Serialize, Deserialize, Debug)]
        pub struct Point {
            pub x: i32,
            pub y: i32,
        }

        #[derive(Serialize, Deserialize, Debug)]
        pub struct Header {
            pub len: i32,
            pub msg_type: String,
        }

        #[derive(Serialize, Deserialize, Debug)]
        pub struct ReturnMessage {
            pub msg_type: String, 
            pub address: i32,
            pub tank_level: u16,
            pub tank_inflow: u16,
        }

        pub fn convert_f32_to_mobdus_u16(_min: f32, max: f32, value: f32) -> u16{
            let max = 65535 as f32 / max;
            (value as f32 * max) as u16
        }
    }
}
