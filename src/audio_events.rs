use alloc::vec::Vec;
use anyhow::Error;
use crankstart::log_to_console;
use crankstart::sound::{AudioSample, SamplePlayer, Sound};
use hashbrown::HashMap;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum AudioEvent {
    MoneyGained,
    UpgradeBought,
    UpgradeDenied,
    DoughCreated,
}

impl AudioEvent {
    const ALL: [AudioEvent; 4] = [
        AudioEvent::MoneyGained,
        AudioEvent::UpgradeBought,
        AudioEvent::UpgradeDenied,
        AudioEvent::DoughCreated,
    ];

    fn to_sound_file(&self) -> &'static str {
        match self {
            AudioEvent::MoneyGained => "res/audio/75235__creek23__cha-ching.wav",
            AudioEvent::UpgradeBought => "res/audio/611800__metalfortress__confirm.wav",
            AudioEvent::UpgradeDenied => "res/audio/220187__gameaudio__loosedeny-casual-1.wav",
            AudioEvent::DoughCreated => {
                "res/audio/330997__rudmer_rotteveel__stick-hitting-a-dreadlock-small-thud.wav"
            }
        }
    }
    fn load_sound(&self) -> Result<AudioSample, Error> {
        let sound = Sound::get();
        sound.load_audio_sample(self.to_sound_file()).map_err(|e| {
            log_to_console!("Failed to load audio sample: {:?}", e);
            e
        })
    }
}

#[derive(Debug)]
pub struct SoundStore {
    sounds: HashMap<AudioEvent, AudioSample>,
    players: Vec<SamplePlayer>,
}

impl SoundStore {
    pub fn new() -> Result<Self, Error> {
        {
            let mut sound = Sound::get();
            sound.set_outputs_active(true, true)?;
        }
        let sounds = AudioEvent::ALL
            .iter()
            .map(|event| event.load_sound().map(|sample| (*event, sample)))
            .collect::<Result<HashMap<AudioEvent, AudioSample>, _>>()?;
        Ok(Self {
            sounds,
            players: Vec::new(),
        })
    }

    fn get_sample(&self, event: &AudioEvent) -> Option<&AudioSample> {
        self.sounds.get(event)
    }
}

#[derive(Debug)]
pub struct AudioEventChannel(Vec<AudioEvent>);

impl AudioEventChannel {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, event: AudioEvent) {
        self.0.push(event);
    }
}

fn play_sample(
    sound: &Sound,
    event: &AudioEvent,
    samples: &HashMap<AudioEvent, AudioSample>,
) -> Result<SamplePlayer, Error> {
    log_to_console!("Playing sample: {:?}", event);
    let sample = samples
        .get(event)
        .ok_or(anyhow::anyhow!("No sample for event"))?;
    let mut player = sound.get_sample_player()?;
    player.set_sample(&sample)?;
    player.play(1, 1.0)?;
    Ok(player)
}
pub fn process_events(channel: &mut AudioEventChannel, sound_store: &mut SoundStore) {
    for event in channel.0.drain(..) {
        let sound = Sound::get();
        match play_sample(&sound, &event, &sound_store.sounds) {
            Ok(player) => sound_store.players.push(player),
            Err(e) => log_to_console!("Failed to play sample: {:?}", e),
        }
    }
    sound_store
        .players
        .retain(|player| player.is_playing().unwrap_or(false));
}
