//! MMLファイル読み取りモジュール
//!
//! `.mml`ファイルからMMLを読み込み、コメント行と空行を除去してMML文字列を返す。
//!
//! # ビジネスルール
//! - BR-067: `.mml`拡張子のみ受け付け
//! - BR-068: UTF-8エンコーディング必須
//! - BR-069: 1MB以下のファイルサイズ制限
//! - BR-070: `#`で始まる行はコメント
//! - BR-071: 空白行は無視

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

/// ファイルサイズ上限（1MB）
const MAX_FILE_SIZE: u64 = 1_000_000;

/// MMLファイルを読み込み、コメントと空行を除去してMML文字列を返す
///
/// # 引数
/// * `path` - ファイルパス
///
/// # 戻り値
/// * `Ok(String)` - MML文字列（コメント、空行除去済み）
/// * `Err(anyhow::Error)` - ファイル読み取りエラー
///
/// # エラー
/// - ファイルが存在しない
/// - 拡張子が`.mml`以外
/// - ファイルサイズが1MB超
/// - UTF-8以外のエンコーディング
/// - ファイルにMMLが含まれていない
///
/// # 例
/// ```ignore
/// use sine_mml::mml::file::read_mml_file;
///
/// let mml = read_mml_file("song.mml")?;
/// println!("MML: {}", mml);
/// ```
pub fn read_mml_file(path: &str) -> Result<String> {
    let path = Path::new(path);

    // ファイル存在確認
    if !path.exists() {
        anyhow::bail!("ファイルが見つかりません: {}", path.display());
    }

    // 拡張子確認 (BR-067)
    if path.extension().and_then(|s| s.to_str()) != Some("mml") {
        anyhow::bail!(
            "ファイル拡張子は .mml である必要があります: {}",
            path.display()
        );
    }

    // ファイルサイズ確認 (BR-069)
    let metadata = fs::metadata(path)?;
    if metadata.len() > MAX_FILE_SIZE {
        anyhow::bail!(
            "ファイルサイズが大きすぎます（上限: 1MB）: {}",
            path.display()
        );
    }

    // ファイル読み込み (BR-068: UTF-8)
    let content = fs::read_to_string(path)
        .with_context(|| format!("ファイルの読み込みに失敗しました: {}", path.display()))?;

    // コメントと空行を除去 (BR-070, BR-071)
    let mml = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect::<Vec<_>>()
        .join(" ");

    if mml.is_empty() {
        anyhow::bail!("ファイルにMMLが含まれていません: {}", path.display());
    }

    Ok(mml)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    // === 正常系テスト ===

    #[test]
    fn test_read_mml_file_success() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.mml");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "CDEFGAB").unwrap();

        let result = read_mml_file(file_path.to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "CDEFGAB");
    }

    #[test]
    fn test_read_mml_file_with_comments() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.mml");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "# Comment").unwrap();
        writeln!(file, "CDEFGAB").unwrap();
        writeln!(file).unwrap();
        writeln!(file, "# Another comment").unwrap();
        writeln!(file, ">C").unwrap();

        let result = read_mml_file(file_path.to_str().unwrap());
        assert!(result.is_ok());
        let mml = result.unwrap();
        assert_eq!(mml, "CDEFGAB >C");
    }

    #[test]
    fn test_read_mml_file_trim_whitespace() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.mml");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "  CDEFGAB  ").unwrap();
        writeln!(file, "  >C  ").unwrap();

        let result = read_mml_file(file_path.to_str().unwrap());
        assert!(result.is_ok());
        let mml = result.unwrap();
        assert_eq!(mml, "CDEFGAB >C");
    }

    #[test]
    fn test_read_mml_file_with_indented_comment() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.mml");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "  # Indented comment").unwrap();
        writeln!(file, "CDEFGAB").unwrap();

        let result = read_mml_file(file_path.to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "CDEFGAB");
    }

    #[test]
    fn test_read_mml_file_complex_content() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.mml");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "# イントロ").unwrap();
        writeln!(file, "T120 L8 O5").unwrap();
        writeln!(file).unwrap();
        writeln!(file, "# Aメロ").unwrap();
        writeln!(file, "CDEFGAB").unwrap();

        let result = read_mml_file(file_path.to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "T120 L8 O5 CDEFGAB");
    }

    #[test]
    fn test_read_mml_file_single_line() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.mml");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "C").unwrap();

        let result = read_mml_file(file_path.to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "C");
    }

    // === 異常系テスト ===

    #[test]
    fn test_read_mml_file_not_found() {
        let result = read_mml_file("nonexistent.mml");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("ファイルが見つかりません"));
    }

    #[test]
    fn test_read_mml_file_invalid_extension() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "CDEFGAB").unwrap();

        let result = read_mml_file(file_path.to_str().unwrap());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("ファイル拡張子は .mml である必要があります"));
    }

    #[test]
    fn test_read_mml_file_no_extension() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("testfile");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "CDEFGAB").unwrap();

        let result = read_mml_file(file_path.to_str().unwrap());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("ファイル拡張子は .mml である必要があります"));
    }

    #[test]
    fn test_read_mml_file_too_large() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("large.mml");
        let mut file = File::create(&file_path).unwrap();
        // 1MB超のファイルを作成（約8バイト * 125,001行 = 約1MB）
        for _ in 0..125_001 {
            writeln!(file, "CDEFGAB").unwrap();
        }

        let result = read_mml_file(file_path.to_str().unwrap());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("ファイルサイズが大きすぎます"));
    }

    #[test]
    fn test_read_mml_file_empty() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("empty.mml");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "# Only comments").unwrap();
        writeln!(file, "").unwrap();

        let result = read_mml_file(file_path.to_str().unwrap());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("ファイルにMMLが含まれていません"));
    }

    #[test]
    fn test_read_mml_file_only_whitespace_lines() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("whitespace.mml");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "   ").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "\t").unwrap();

        let result = read_mml_file(file_path.to_str().unwrap());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("ファイルにMMLが含まれていません"));
    }

    // === エッジケーステスト ===

    #[test]
    fn test_read_mml_file_hash_in_middle_of_line() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.mml");
        let mut file = File::create(&file_path).unwrap();
        // '#' が行の途中にある場合（C#など）はコメントとして扱わない
        writeln!(file, "C# D E").unwrap();

        let result = read_mml_file(file_path.to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "C# D E");
    }

    #[test]
    fn test_read_mml_file_only_hash() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.mml");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "#").unwrap();
        writeln!(file, "CDE").unwrap();

        let result = read_mml_file(file_path.to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "CDE");
    }
}
