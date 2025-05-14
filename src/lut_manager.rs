use std::collections::HashMap;
use std::io;

#[derive(Debug, Clone)]
pub struct LutData {
    pub name: String,
    pub red: Vec<u8>,
    pub green: Vec<u8>,
    pub blue: Vec<u8>,
}

impl LutData {
    pub fn reverse(&mut self) {
        self.red.reverse();
        self.green.reverse();
        self.blue.reverse();
    }
}

macro_rules! include_luts {
    ($($name:expr),*) => {
        {
            let mut map = HashMap::new();
            $(
                map.insert(
                    $name.strip_suffix(".lut").unwrap(),
                    include_bytes!(concat!("../luts/", $name))
                );
            )*
            map
        }
    };
}

lazy_static::lazy_static! {
    static ref EMBEDDED_LUTS: HashMap<&'static str, &'static [u8; 768]> = include_luts!(
        "MATPLOTLIB_Accent_r.lut",
        "MATPLOTLIB_bone_r.lut",
        "MATPLOTLIB_brg_r.lut",
        "MATPLOTLIB_bwr_r.lut",
        "MATPLOTLIB_cool_r.lut",
        "MATPLOTLIB_coolwarm_r.lut",
        "MATPLOTLIB_copper_r.lut",
        "MATPLOTLIB_cubehelix_r.lut",
        "MATPLOTLIB_Dark2_r.lut",
        "MATPLOTLIB_flag_r.lut",
        "MATPLOTLIB_gist_earth_r.lut",
        "MATPLOTLIB_gist_gray_r.lut",
        "MATPLOTLIB_gist_grey_r.lut",
        "MATPLOTLIB_gist_heat_r.lut",
        "MATPLOTLIB_gist_ncar_r.lut",
        "MATPLOTLIB_gist_rainbow_r.lut",
        "MATPLOTLIB_gist_stern_r.lut",
        "MATPLOTLIB_gist_yarg_r.lut",
        "MATPLOTLIB_gist_yerg_r.lut",
        "MATPLOTLIB_gnuplot_r.lut",
        "MATPLOTLIB_gnuplot2_r.lut",
        "MATPLOTLIB_gray_r.lut",
        "MATPLOTLIB_Grays_r.lut",
        "MATPLOTLIB_grey_r.lut",
        "MATPLOTLIB_hot_r.lut",
        "MATPLOTLIB_hsv_r.lut",
        "MATPLOTLIB_jet_r.lut",
        "MATPLOTLIB_nipy_spectral_r.lut",
        "MATPLOTLIB_ocean_r.lut",
        "MATPLOTLIB_Paired_r.lut",
        "MATPLOTLIB_Pastel1_r.lut",
        "MATPLOTLIB_Pastel2_r.lut",
        "MATPLOTLIB_pink_r.lut",
        "MATPLOTLIB_prism_r.lut",
        "MATPLOTLIB_rainbow_r.lut",
        "MATPLOTLIB_seismic_r.lut",
        "MATPLOTLIB_Set1_r.lut",
        "MATPLOTLIB_Set2_r.lut",
        "MATPLOTLIB_Set3_r.lut",
        "MATPLOTLIB_spring_r.lut",
        "MATPLOTLIB_summer_r.lut",
        "MATPLOTLIB_tab10_r.lut",
        "MATPLOTLIB_tab20_r.lut",
        "MATPLOTLIB_tab20b_r.lut",
        "MATPLOTLIB_tab20c_r.lut",
        "MATPLOTLIB_terrain_r.lut",
        "MATPLOTLIB_winter_r.lut"
        // Note: Add other LUTs as needed
    );
}

pub struct LutManager;

impl LutManager {
    pub fn new() -> Self {
        Self
    }

    pub fn get_available_luts(&self) -> Vec<String> {
        EMBEDDED_LUTS.keys().map(|&name| name.to_string()).collect()
    }

    pub fn load_lut(&self, name: &str) -> io::Result<LutData> {
        let buffer = EMBEDDED_LUTS.get(name).ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, format!("LUT '{}' not found", name))
        })?;

        // Each color component should be 256 bytes
        if buffer.len() != 768 {
            // 256 * 3 (RGB)
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid LUT file size",
            ));
        }

        let red = buffer[0..256].to_vec();
        let green = buffer[256..512].to_vec();
        let blue = buffer[512..768].to_vec();

        Ok(LutData {
            name: name.to_string(),
            red,
            green,
            blue,
        })
    }
}

impl Default for LutManager {
    fn default() -> Self {
        Self::new()
    }
}
