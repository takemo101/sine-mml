use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum AudioError {
    #[error("[AUD-E001] オーディオデバイスが見つかりませんでした")]
    DeviceNotFound,
    #[error("[AUD-E002] オーディオストリームの作成に失敗しました: {0}")]
    StreamCreationError(String),
    #[error("[AUD-E003] 再生中にエラーが発生しました: {0}")]
    PlaybackError(String),
    #[error("[AUD-E004] WAVファイルの書き込みに失敗しました: {0}")]
    WavWriteError(String),
    #[error("[AUD-E005] 無効なパラメータ '{parameter}': {reason}")]
    InvalidParameter { parameter: String, reason: String },
}
