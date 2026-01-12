#[derive(Debug, Clone, PartialEq)]
pub struct Mml {
    pub commands: Vec<Command>,
}

impl Mml {
    /// MMLコマンドから最初に設定されたテンポを取得する。
    /// Tempoコマンドがない場合はデフォルトの120を返す。
    #[must_use]
    pub fn get_tempo(&self) -> u16 {
        for command in &self.commands {
            if let Command::Tempo(tempo) = command {
                return tempo.value;
            }
        }
        120
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Note(Note),
    Rest(Rest),
    Octave(Octave),
    OctaveUp,
    OctaveDown,
    Tempo(Tempo),
    DefaultLength(DefaultLength),
    Volume(Volume),
    /// ループコマンド: [commands]n
    ///
    /// # フィールド
    /// - `commands`: ループ内のコマンド列
    /// - `escape_index`: 脱出ポイントのインデックス（:の位置）
    ///   - `None`: 脱出ポイントなし（全コマンドを繰り返す）
    ///   - `Some(n)`: n番目のコマンドの後で最終回に脱出
    /// - `repeat_count`: ループ回数（1-99）
    Loop {
        /// ループ内のコマンド列
        commands: Vec<Command>,
        /// 脱出ポイントのインデックス（Noneの場合は脱出なし）
        escape_index: Option<usize>,
        /// 繰り返し回数（1-99）
        repeat_count: usize,
    },
}

/// 単一の音価を表現
///
/// # フィールド
/// - `value`: 音価（1=全音符, 2=2分音符, 4=4分音符, etc.）、Noneの場合はデフォルト音長を使用
/// - `dots`: 付点の数（0-3）
#[derive(Debug, Clone, PartialEq)]
pub struct Duration {
    /// 音価（None の場合はデフォルト音長を使用）
    pub value: Option<u8>,
    /// 付点の数
    pub dots: u8,
}

impl Duration {
    /// Creates a new `Duration`
    #[must_use]
    pub const fn new(value: Option<u8>, dots: u8) -> Self {
        Self { value, dots }
    }

    /// 秒単位での音長を計算
    #[must_use]
    pub fn duration_in_seconds(&self, bpm: u16, default_length: u8) -> f32 {
        let length = f32::from(self.value.unwrap_or(default_length));
        if length == 0.0 {
            return 0.0;
        }
        let base_duration = 240.0 / (f32::from(bpm) * length);
        let dot_multiplier = calculate_dot_multiplier(self.dots);
        base_duration * dot_multiplier
    }
}

/// タイで連結された音価を表現
///
/// # フィールド
/// - `base`: ベース音価（最初の音符の音価）
/// - `tied`: 連結される追加音価のリスト
///
/// # 例
/// ```ignore
/// // C4&8 の場合
/// TiedDuration {
///     base: Duration { value: Some(4), dots: 0 },
///     tied: vec![Duration { value: Some(8), dots: 0 }],
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct TiedDuration {
    /// ベース音価（最初の音符の音価）
    pub base: Duration,
    /// 連結される追加音価のリスト
    pub tied: Vec<Duration>,
}

impl TiedDuration {
    /// Creates a new `TiedDuration`
    #[must_use]
    pub fn new(base: Duration) -> Self {
        Self {
            base,
            tied: Vec::new(),
        }
    }

    /// タイで連結する音価を追加
    pub fn add_tie(&mut self, duration: Duration) {
        self.tied.push(duration);
    }

    /// タイが存在するかどうか
    #[must_use]
    pub fn has_ties(&self) -> bool {
        !self.tied.is_empty()
    }

    /// 合計音長を秒単位で計算
    #[must_use]
    pub fn total_duration_in_seconds(&self, bpm: u16, default_length: u8) -> f32 {
        let mut total = self.base.duration_in_seconds(bpm, default_length);
        for tied in &self.tied {
            total += tied.duration_in_seconds(bpm, default_length);
        }
        total
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    pub pitch: Pitch,
    pub accidental: Accidental,
    pub duration: Option<u8>,
    pub dots: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pitch {
    C = 0,
    D = 2,
    E = 4,
    F = 5,
    G = 7,
    A = 9,
    B = 11,
}

impl Pitch {
    #[must_use]
    pub fn from_char(c: char) -> Option<Self> {
        match c.to_ascii_uppercase() {
            'C' => Some(Self::C),
            'D' => Some(Self::D),
            'E' => Some(Self::E),
            'F' => Some(Self::F),
            'G' => Some(Self::G),
            'A' => Some(Self::A),
            'B' => Some(Self::B),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Accidental {
    Natural = 0,
    Sharp = 1,
    Flat = -1,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rest {
    pub duration: Option<u8>,
    pub dots: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Octave {
    pub value: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tempo {
    pub value: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefaultLength {
    pub value: u8,
}

/// ボリューム値の種類
///
/// # バリアント
/// - `Absolute(u8)`: 絶対値（0-15）
/// - `Relative(i8)`: 相対値（-15〜+15）
///
/// # 例
/// ```ignore
/// // V10 の場合
/// VolumeValue::Absolute(10)
///
/// // V+2 の場合
/// VolumeValue::Relative(2)
///
/// // V-3 の場合
/// VolumeValue::Relative(-3)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumeValue {
    /// 絶対値（0-15）
    Absolute(u8),
    /// 相対値（-15〜+15）
    Relative(i8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Volume {
    /// 絶対値（0-15）または相対値（-15〜+15）
    pub value: VolumeValue,
}

impl Note {
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn to_midi_note(&self, octave: u8) -> u8 {
        let base_note = self.pitch as i16;
        let accidental_offset = self.accidental as i16;
        let midi_note = (i16::from(octave) + 1) * 12 + base_note + accidental_offset;
        midi_note.clamp(0, 127) as u8
    }

    #[must_use]
    pub fn duration_in_seconds(&self, bpm: u16, default_length: u8) -> f32 {
        let length = f32::from(self.duration.unwrap_or(default_length));
        if length == 0.0 {
            return 0.0;
        }
        let base_duration = 240.0 / (f32::from(bpm) * length);
        let dot_multiplier = calculate_dot_multiplier(self.dots);
        base_duration * dot_multiplier
    }
}

impl Rest {
    #[must_use]
    pub fn duration_in_seconds(&self, bpm: u16, default_length: u8) -> f32 {
        let length = f32::from(self.duration.unwrap_or(default_length));
        if length == 0.0 {
            return 0.0;
        }
        let base_duration = 240.0 / (f32::from(bpm) * length);
        let dot_multiplier = calculate_dot_multiplier(self.dots);
        base_duration * dot_multiplier
    }
}

fn calculate_dot_multiplier(dots: u8) -> f32 {
    match dots {
        0 => 1.0,
        1 => 1.5,
        2 => 1.75,
        3 => 1.875,
        _ => 1.0 + (1.0 - 0.5_f32.powi(i32::from(dots))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pitch_from_char_valid() {
        assert_eq!(Pitch::from_char('C'), Some(Pitch::C));
        assert_eq!(Pitch::from_char('c'), Some(Pitch::C));
        assert_eq!(Pitch::from_char('G'), Some(Pitch::G));
    }

    #[test]
    fn pitch_from_char_invalid() {
        assert_eq!(Pitch::from_char('X'), None);
        assert_eq!(Pitch::from_char('H'), None);
    }

    #[test]
    fn note_to_midi_c4_equals_60() {
        let note = Note {
            pitch: Pitch::C,
            accidental: Accidental::Natural,
            duration: None,
            dots: 0,
        };
        assert_eq!(note.to_midi_note(4), 60);
    }

    #[test]
    fn note_to_midi_a4_equals_69() {
        let note = Note {
            pitch: Pitch::A,
            accidental: Accidental::Natural,
            duration: None,
            dots: 0,
        };
        assert_eq!(note.to_midi_note(4), 69);
    }

    #[test]
    fn note_duration_quarter_at_120bpm() {
        let note = Note {
            pitch: Pitch::C,
            accidental: Accidental::Natural,
            duration: Some(4),
            dots: 0,
        };
        let duration = note.duration_in_seconds(120, 4);
        assert!((duration - 0.5).abs() < 0.001);
    }

    #[test]
    fn note_duration_dotted() {
        let note = Note {
            pitch: Pitch::C,
            accidental: Accidental::Natural,
            duration: Some(4),
            dots: 1,
        };
        let duration = note.duration_in_seconds(120, 4);
        assert!((duration - 0.75).abs() < 0.001);
    }

    #[test]
    fn get_tempo_with_tempo_command() {
        let mml = Mml {
            commands: vec![
                Command::Tempo(Tempo { value: 180 }),
                Command::Note(Note {
                    pitch: Pitch::C,
                    accidental: Accidental::Natural,
                    duration: Some(4),
                    dots: 0,
                }),
            ],
        };
        assert_eq!(mml.get_tempo(), 180);
    }

    #[test]
    fn get_tempo_without_tempo_command_returns_default() {
        let mml = Mml {
            commands: vec![Command::Note(Note {
                pitch: Pitch::C,
                accidental: Accidental::Natural,
                duration: Some(4),
                dots: 0,
            })],
        };
        assert_eq!(mml.get_tempo(), 120);
    }

    // TiedDuration tests
    #[test]
    fn tied_duration_new() {
        let duration = TiedDuration::new(Duration::new(Some(4), 0));
        assert_eq!(duration.base.value, Some(4));
        assert_eq!(duration.base.dots, 0);
        assert!(duration.tied.is_empty());
    }

    #[test]
    fn tied_duration_add_tie() {
        let mut duration = TiedDuration::new(Duration::new(Some(4), 0));
        duration.add_tie(Duration::new(Some(8), 0));
        assert_eq!(duration.tied.len(), 1);
        assert_eq!(duration.tied[0].value, Some(8));
    }

    #[test]
    fn tied_duration_has_ties() {
        let mut duration = TiedDuration::new(Duration::new(Some(4), 0));
        assert!(!duration.has_ties());
        duration.add_tie(Duration::new(Some(8), 0));
        assert!(duration.has_ties());
    }

    #[test]
    fn tied_duration_total_duration_simple() {
        // C4&8 at 120 BPM: 4分音符(0.5s) + 8分音符(0.25s) = 0.75s
        let mut duration = TiedDuration::new(Duration::new(Some(4), 0));
        duration.add_tie(Duration::new(Some(8), 0));
        let total = duration.total_duration_in_seconds(120, 4);
        assert!((total - 0.75).abs() < 0.001);
    }

    #[test]
    fn tied_duration_total_duration_multiple_ties() {
        // C4&8&16 at 120 BPM: 4分音符(0.5s) + 8分音符(0.25s) + 16分音符(0.125s) = 0.875s
        let mut duration = TiedDuration::new(Duration::new(Some(4), 0));
        duration.add_tie(Duration::new(Some(8), 0));
        duration.add_tie(Duration::new(Some(16), 0));
        let total = duration.total_duration_in_seconds(120, 4);
        assert!((total - 0.875).abs() < 0.001);
    }

    #[test]
    fn duration_in_seconds_with_default() {
        // デフォルト音長が4の場合、Noneは4分音符として扱われる
        let duration = Duration::new(None, 0);
        let seconds = duration.duration_in_seconds(120, 4);
        assert!((seconds - 0.5).abs() < 0.001);
    }

    #[test]
    fn duration_in_seconds_with_dots() {
        // 4分付点音符 at 120 BPM: 0.5s * 1.5 = 0.75s
        let duration = Duration::new(Some(4), 1);
        let seconds = duration.duration_in_seconds(120, 4);
        assert!((seconds - 0.75).abs() < 0.001);
    }
}
