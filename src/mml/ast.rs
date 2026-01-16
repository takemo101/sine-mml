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
    /// 連符コマンド: `{commands}n[:base_duration]`
    ///
    /// # フィールド
    /// - `commands`: 連符内のコマンド列（`Note`, `Rest`, ネストした`Tuplet`）
    /// - `count`: 連符数（2-99）
    /// - `base_duration`: ベース音長（`:` の後の指定、`None`の場合はデフォルト音長を使用）
    ///
    /// # 例
    /// ```ignore
    /// // {CDE}3
    /// Tuplet {
    ///     commands: vec![Note(C), Note(D), Note(E)],
    ///     count: 3,
    ///     base_duration: None,
    /// }
    ///
    /// // {CDE}3:2
    /// Tuplet {
    ///     commands: vec![Note(C), Note(D), Note(E)],
    ///     count: 3,
    ///     base_duration: Some(2),
    /// }
    /// ```
    Tuplet {
        /// 連符内のコマンド列
        commands: Vec<Command>,
        /// 連符数（2-99）
        count: u8,
        /// ベース音長（`None`の場合はデフォルト音長を使用）
        base_duration: Option<u8>,
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

    /// 拍数単位での音長を計算
    ///
    /// # Arguments
    /// * `default_duration` - デフォルト音価（省略時）
    ///
    /// # Returns
    /// 音長（拍数）。4分音符 = 1拍として計算。
    ///
    /// # Examples
    /// - 4分音符 (value=4) → 1.0拍
    /// - 2分音符 (value=2) → 2.0拍
    /// - 8分音符 (value=8) → 0.5拍
    /// - 4分付点 (value=4, dots=1) → 1.5拍
    #[must_use]
    pub fn to_beats(&self, default_duration: u8) -> f64 {
        let length = f64::from(self.value.unwrap_or(default_duration));
        if length == 0.0 {
            return 0.0;
        }
        // 4分音符を1拍とする計算: 4 / length
        let base_beats = 4.0 / length;
        let dot_multiplier = f64::from(calculate_dot_multiplier(self.dots));
        base_beats * dot_multiplier
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

    /// 総音長を拍数（f64）で計算
    ///
    /// # Arguments
    /// * `default_duration` - デフォルト音価（省略時）
    ///
    /// # Returns
    /// 総音長（拍数）。4分音符 = 1拍として計算。
    ///
    /// # Examples
    /// - `C4&8` (default=4): 1.0拍 + 0.5拍 = 1.5拍
    /// - `C4&8&16` (default=4): 1.0拍 + 0.5拍 + 0.25拍 = 1.75拍
    /// - `C4.&8` (default=4): 1.5拍 + 0.5拍 = 2.0拍
    #[must_use]
    pub fn total_beats(&self, default_duration: u8) -> f64 {
        let base_beats = self.base.to_beats(default_duration);
        let tied_beats: f64 = self.tied.iter().map(|d| d.to_beats(default_duration)).sum();
        base_beats + tied_beats
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
    pub duration: TiedDuration,
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
    pub duration: TiedDuration,
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

    /// 音符の総音長を拍数で取得
    ///
    /// # Arguments
    /// * `default_duration` - デフォルト音価（省略時）
    ///
    /// # Returns
    /// 総音長（拍数）。4分音符 = 1拍として計算。
    #[must_use]
    pub fn total_beats(&self, default_duration: u8) -> f64 {
        self.duration.total_beats(default_duration)
    }

    #[must_use]
    pub fn duration_in_seconds(&self, bpm: u16, default_length: u8) -> f32 {
        self.duration.total_duration_in_seconds(bpm, default_length)
    }
}

impl Rest {
    /// 休符の総音長を拍数で取得
    ///
    /// # Arguments
    /// * `default_duration` - デフォルト音価（省略時）
    ///
    /// # Returns
    /// 総音長（拍数）。4分音符 = 1拍として計算。
    #[must_use]
    pub fn total_beats(&self, default_duration: u8) -> f64 {
        self.duration.total_beats(default_duration)
    }

    #[must_use]
    pub fn duration_in_seconds(&self, bpm: u16, default_length: u8) -> f32 {
        self.duration.total_duration_in_seconds(bpm, default_length)
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
