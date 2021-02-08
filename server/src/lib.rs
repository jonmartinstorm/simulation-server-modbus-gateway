pub mod utils {
    
    pub mod watertank {
        use serde::{Serialize, Deserialize};
        use rand::prelude::*;
        use rand_distr::{Normal, Distribution};

        // How many cubic MM in one liter, Thousand liters in one cubic M. 
        const L_TO_CUBIC_MM: i64 = 1000000;
        /// A water tank struct
        /// There is flow into the water tank a size of the water tank and a level we want the water to be
        /// There is an areal of the water tank, it is a box watertank with a hight
        /// There is also a valve out of the water tank that controls the outflow
        /// 
        #[derive(Serialize, Deserialize, Debug, Copy, Clone)]
        pub struct WaterTank {
            pub level: i64,         // the water level of the tank mm. 
            pub inflow_mean: f32,   // the mean inflow if the tank l/s
            pub inflow_stddev: f32, // the stddev of inflow of the tank l/s
            pub inflow: f64,        // the inflow right now
            pub areal: i64,         // the areal of the tank mm^2
            pub height: i64,        // the height of the tank mm
            pub outflow: f64,       // the outflow of the tank l/s
            pub set_level: i64,     // the wanted level of the tank mm, Real value? or 4 - 20 mA?
        }
    
        impl WaterTank {
            fn _volume(&self) -> i64 {
                self.areal * self.height
            }
    
            pub fn update_level(&mut self, seconds_passed: f32) {
                // water volume of the tank = areal * level
                // change in volume = volume + (inflow - outflow) * seconds_passed
                let volume = (self.areal * self.level) as f64 + ((self.inflow - self.outflow) * seconds_passed as f64 * L_TO_CUBIC_MM as f64);
                self.level = (volume / self.areal as f64) as i64;
            }
    
            pub fn update_inflow(&mut self) {
                let mut rng = thread_rng();
                let normal = Normal::new(self.inflow_mean, self.inflow_stddev).unwrap();
                let v = normal.sample(&mut rng);
                self.inflow = v as f64;
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
    }
}
