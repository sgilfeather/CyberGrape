use std::f64::consts::PI;

use crate::hdm::Hdm;
use crate::saf::BufferMetadata;
use crate::update_accumulator::UpdateAccumulator;

/* IDs of our particular antennas and tags

   Back antenna (base) = 118875763481542
   Front antenna (aux) = 118875763481510

   Tag 1 = 118875764010724
   Tag 2 = 118875764011634
*/

const BACK_ANTENNA: usize = 118875763481542;
const FRONT_ANTENNA: usize = 118875763481510;

// A tuple of the gain and range of the tags
type TagSetting = (f32, f32);

pub struct Sphericalizer {
    tag_settings: Vec<TagSetting>,
}

impl Sphericalizer {
    pub fn new(tag_settings: Vec<TagSetting>) -> Self {
        Self { tag_settings }
    }

    // From observation, azimuth and elevation are in the range of -70 to 70 degrees (-1.22173 to 1.22173 rad)
    // This function scales them to the range -90 to 90 degrees (-PI/2 to PI/2 rad)
    fn scale_angle(azm: f64) -> f64 {
        let pi_2 = PI / 2.0;
        let scaled = azm * pi_2 / 1.22173;
        scaled.clamp(-pi_2, pi_2)
    }

    pub fn query(&self, acc: &mut UpdateAccumulator<Hdm>) -> Option<Vec<BufferMetadata>> {
        let mut updates = acc.get_status();
        // There should be two updates for each tag since there are two antennas
        // If there are not, then we must wait until more updates come in
        if updates.len() != self.tag_settings.len() * 2 {
            return None;
        }
        // Sort by tag ID
        updates.sort_by(|a, b| a.dst.cmp(&b.dst));
        // Group updates into pairs, one for each tag, where each pair is from the back and front antennas
        let grouped_updates = updates.chunks(2);
        // For each pair, derive a single BufferMetadata
        grouped_updates
            .enumerate()
            .map(|(i, pair)| {
                let back_ant = pair
                    .iter()
                    .find(|u| u.src == BACK_ANTENNA)
                    .expect("Missing an update from the back antenna");
                let front_ant = pair
                    .iter()
                    .find(|u| u.src == FRONT_ANTENNA)
                    .expect("Missing an update from the front antenna");
                let (gain, range) = self.tag_settings[i];
                let mut metadata = BufferMetadata {
                    azimuth: Sphericalizer::scale_angle(back_ant.azm) as f32,
                    elevation: Sphericalizer::scale_angle(back_ant.elv) as f32,
                    range: range,
                    gain: gain,
                };
                // The front antenna informs whether the tag is in front or behind the base antenna, since the base itself cannot tell
                if front_ant.azm < 0.0 {
                    metadata.azimuth = PI as f32 - metadata.azimuth;
                };
                metadata
            })
            .collect::<Vec<_>>()
            .into()
    }
}
