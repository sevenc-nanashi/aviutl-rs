use super::sys;
use crate::AnyResult;
use image::RgbImage;

use std::path::Path;

use derive_getters::Getters;
use num_rational::Rational32;

#[derive(Debug, Getters)]
/// 出力情報の構造体。OUTPUT_INFOと対応。
pub struct OutputInfo<'a> {
    #[getter(skip)]
    inner: &'a sys::OutputInfo,

    /// 出力フラグ。
    flag: OutputInfoFlag,
    /// 幅。
    w: u32,
    /// 高さ。
    h: u32,
    /// フレーム数。
    n: u32,
    /// フレームレート。
    frame_rate: Rational32,
    /// 1フレームのバイト数。
    size: u32,
    /// サンプリングレート。
    audio_rate: u32,
    /// チャンネル数。
    audio_ch: u32,
    /// サンプル数。
    audio_n: u32,
    /// 1サンプルのバイト数。
    audio_size: u32,
    /// 保存ファイル名。
    savefile: &'a Path,
}

impl OutputInfo<'_> {
    /// 特定のフレームの画像を取得する。
    ///
    /// <div class="warning">
    /// この関数は、[`Self::update_preview()`]の呼び出しなどを行わないため、
    /// 基本的には、[`Self::frames()`]を使うこと。
    /// </div>
    pub fn get_video(&self, frame: i32) -> AnyResult<RgbImage> {
        let ptr = (self.inner.func_get_video)(frame);
        let slice = unsafe { std::slice::from_raw_parts(ptr, self.size as _) }.to_vec();
        let frame = RgbImage::from_fn(self.w as _, self.h as _, |x, y| {
            let i = (x + (self.h - y - 1) * self.w) as usize;
            image::Rgb([slice[i * 3 + 2], slice[i * 3 + 1], slice[i * 3]])
        });

        Ok(frame)
    }

    /// フレームの画像をIteratorとして取得する。
    pub fn frames(&self) -> Frames<'_> {
        Frames::new(self)
    }

    /// プレビュー画面を更新する。
    pub fn update_preview(&self) -> AnyResult<()> {
        let result = (self.inner.func_update_preview)();
        if result == 0 {
            return Err(anyhow::anyhow!("Preview update failed"));
        }

        Ok(())
    }

    /// 中断するべきかどうかを取得する。
    pub fn should_abort(&self) -> bool {
        (self.inner.func_is_abort)() != 0
    }

    /// 残り時間を表示する。
    pub fn rest_time_disp(&self, now: i32, total: i32) -> AnyResult<()> {
        let result = (self.inner.func_rest_time_disp)(now, total);
        if result == 0 {
            return Err(anyhow::anyhow!("Rest time display failed"));
        }

        Ok(())
    }
}

/// フレームの画像をIteratorとして取得する。
///
/// 中断された時、[`Self::next()`]は[`None`]を返す。
/// このIteratorが終了した後に[`OutputInfo::should_abort()`]で中断されたかどうかを確認する必要がある。
///
/// <div class="warning">
/// [`Self::next()`]には、[`OutputInfo::update_preview()`]や[`OutputInfo::rest_time_disp()`]を呼び出す処理が含まれている。
/// そのため、[`Iterator::to_vec()`]などで、このIteratorを消費すると、
/// [`OutputInfo::update_preview()`]や[`OutputInfo::rest_time_disp()`]が呼び出されなくなり、
/// AviUtlが応答無しの状態になる。
/// </div>
pub struct Frames<'a> {
    info: &'a OutputInfo<'a>,
    frame: i32,
    last_time: std::time::Instant,
}

impl<'a> Frames<'a> {
    fn new(info: &'a OutputInfo<'a>) -> Self {
        let last_time = std::time::Instant::now();
        Self {
            info,
            frame: 0,
            last_time,
        }
    }
}

impl Iterator for Frames<'_> {
    type Item = (RgbImage, f64);

    fn next(&mut self) -> Option<Self::Item> {
        if self.frame >= self.info.n as _ {
            return None;
        }
        if self.last_time.elapsed().as_millis() >= 1000 {
            self.last_time = std::time::Instant::now();
            if self.info.should_abort() {
                return None;
            }
            self.info.update_preview().unwrap();
            self.info
                .rest_time_disp(self.frame, self.info.n as _)
                .unwrap();
        }
        let frame = self.info.get_video(self.frame).unwrap();
        self.frame += 1;
        let time = Rational32::new(self.frame, 1) / self.info.frame_rate;
        Some((frame, (*time.numer() as f64) / (*time.denom() as f64)))
    }
}

#[derive(Debug, Getters)]
/// OUTPUT_INFOのflagフィールドの値を表すフラグ。
pub struct OutputInfoFlag {
    video: bool,
    audio: bool,
    batch: bool,
}

impl From<i32> for OutputInfoFlag {
    fn from(flag: i32) -> Self {
        Self {
            video: flag & sys::OUTPUT_INFO_FLAG_VIDEO != 0,
            audio: flag & sys::OUTPUT_INFO_FLAG_AUDIO != 0,
            batch: flag & sys::OUTPUT_INFO_FLAG_BATCH != 0,
        }
    }
}

impl<'a> From<&'a sys::OutputInfo> for OutputInfo<'a> {
    fn from(info: &'a sys::OutputInfo) -> Self {
        let savefile_str = unsafe {
            std::ffi::CStr::from_ptr(info.savefile as _)
                .to_str()
                .unwrap()
        };
        Self {
            inner: info,
            flag: OutputInfoFlag::from(info.flag),
            w: info.w as _,
            h: info.h as _,
            n: info.n as _,
            frame_rate: Rational32::new(info.rate, info.scale),
            size: info.size as _,
            audio_rate: info.audio_rate as _,
            audio_ch: info.audio_ch as _,
            audio_n: info.audio_n as _,
            audio_size: info.audio_size as _,
            savefile: Path::new(savefile_str),
        }
    }
}
