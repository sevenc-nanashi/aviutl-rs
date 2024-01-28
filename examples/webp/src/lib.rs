use aviutl_rs::{
    output::{
        get_config, set_config, FileFilterEntry, OutputInfo, OutputPluginTable,
        OutputPluginTableInfo, HINSTANCE, HWND,
    },
    register_output_plugin, AnyResult,
};
use serde::{Deserialize, Serialize};
use sprintf::vsprintf;

struct WebpOutput();

slint::slint! {
import { StandardButton, VerticalBox, LineEdit } from "std-widgets.slint";

export global Global {
    callback quit();
    in-out property <string> format;
    in-out property <int> quality;
}

export component ConfigDialog inherits Dialog {
    VerticalBox {
        alignment: start;
        Text {
            text: "WebP出力設定";
            font-size: 24px;
        }
        Text {
            text: "連番ファイル名の付け方 (printf形式)";
            font-size: 12px;
        }
        LineEdit {
            placeholder-text: "_%04d";
            text: Global.format;
        }
        Text {
            text: "webpの品質";
            font-size: 12px;
        }
        LineEdit {
            placeholder-text: "100";
            text: Global.quality;
        }
        HorizontalLayout {
            alignment: center;
            StandardButton {
                kind: ok;
                clicked => {
                    Global.quit()
                }
            }
        }
    }
}
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Config {
    format: String,
    quality: u8,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            format: "_%04d".to_string(),
            quality: 100,
        }
    }
}
thread_local! {
    static CONFIG_DIALOG: ConfigDialog = ConfigDialog::new().unwrap();
}

impl OutputPluginTable for WebpOutput {
    fn info(&self) -> OutputPluginTableInfo {
        OutputPluginTableInfo {
            name: "WebP",
            filefilter: &[FileFilterEntry {
                name: "WebP画像",
                filter: "*.webp",
            }],
            information: "WebP",
            has_config: true,
        }
    }
    fn output(&self, info: &OutputInfo) -> AnyResult<()> {
        let save_dir = info.savefile().parent().unwrap();
        let config: Config = get_config().unwrap_or_default();
        let basename = info.savefile().file_stem().unwrap().to_str().unwrap();
        for (i, (frame, _)) in info.frames().enumerate() {
            let inc_section = vsprintf(&config.format, &[&(i as i32)]).unwrap();
            let path = save_dir.join(format!("{}{}.webp", basename, inc_section));
            let img = frame
                .pixels()
                .flat_map(|p| [p[0], p[1], p[2], 255])
                .collect::<Vec<_>>();
            let (width, height) = frame.dimensions();
            let data = webp::Encoder::new(&img, webp::PixelLayout::Rgba, width, height)
                .encode(config.quality as _)
                .to_vec();
            std::fs::write(path, data)?;
        }

        Ok(())
    }
    fn config(&self, _hwnd: HWND, _dll_hinst: HINSTANCE) {
        let mut config = get_config();
        let config: Config = config.take().unwrap_or_default();
        let old_config = config.clone();
        let mut new_config = CONFIG_DIALOG.with(|config_dialog| {
            config_dialog
                .global::<Global>()
                .set_format(config.format.into());
            config_dialog
                .global::<Global>()
                .set_quality(config.quality as _);
            config_dialog.global::<Global>().on_quit(|| {
                CONFIG_DIALOG.with(|config_dialog| {
                    config_dialog.hide().unwrap();
                })
            });
            config_dialog.run().unwrap();

            let format = config_dialog.global::<Global>().get_format();
            let quality = config_dialog.global::<Global>().get_quality();
            Config {
                format: format.into(),
                quality: quality as _,
            }
        });
        if vsprintf(&new_config.format, &[&0]).is_err()
            || vsprintf(&new_config.format, &[&0]).unwrap()
                == vsprintf(&old_config.format, &[&1]).unwrap()
        {
            new_config.format = old_config.format;
        }
        if new_config.quality < 1 {
            new_config.quality = 1;
        } else if new_config.quality > 100 {
            new_config.quality = 100;
        }
        set_config(&new_config);
    }
}

register_output_plugin!(WebpOutput);
