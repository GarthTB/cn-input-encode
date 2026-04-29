use std::{error, fs, io, io::Write, path::Path};

mod config;
mod cost_map;
mod dp_encoder;
mod layout_map;
mod trie_dict;
mod usage_analyzer;
mod utils;

pub(crate) const CHUNK_SIZE: usize = 1 << 18; // 爆栈风险

pub(crate) type DynResult<T> = Result<T, Box<dyn error::Error>>;

fn main() -> DynResult<()> {
    println!(
        "cn-input-encode 1.0.0 (20260502)\n\
        作者：Garth TB | 天卜 <g-art-h@outlook.com>\n\
        仓库：https://github.com/GarthTB/cn-input-encode\n\
        加载配置..."
    );
    let config = config::Config::load()?;
    let mut encoder = dp_encoder::Encoder::new(&config.dict);
    let mut oo = fs::OpenOptions::new();
    let opener = oo.append(true).create(true);
    println!("...加载完成");

    for i_path in config.text {
        let o_path = gen_o_path(&i_path)?;
        let mut o_file = opener.open(&o_path)?;

        println!("开始编码文件'{i_path}'...");
        encoder.prepare();
        let mut t_len = 0;
        utils::for_each_chunk(&i_path, |s| {
            encoder.shrink();
            let delta = encoder.feed(s);
            print!("\r    第{}-{}字...", t_len + 1, t_len + delta);
            io::stdout().flush()?;
            t_len += delta;
            encoder.proc_chunk(&config.costs, true)?;
            Ok(o_file.write_all(&encoder.build_encoding())?)
        })?;
        encoder.shrink();
        encoder.proc_chunk(&config.costs, false)?;

        let cost = encoder.proc_end(&config.costs)?;
        o_file.write_all(&encoder.build_encoding())?;
        println!("完成，共{t_len}字");

        println!("分析数据...");
        let report = usage_analyzer::analyze(&o_path, t_len, cost, &config.layout)?;
        o_file.write_all(&report)?;
        println!("...完成，结果已写入'{o_path}'");
    }

    Ok(println!("全部完成，程序结束"))
}

fn gen_o_path(i_path: &str) -> DynResult<String> {
    let p = Path::new(&i_path);
    let stem = p.file_stem().unwrap_or_default().to_string_lossy();
    let dir = p.parent().unwrap_or(Path::new("."));
    let mut o_path = dir.join(format!("{stem}-code.txt"));
    let mut n = 2u32;
    while fs::exists(&o_path)? {
        o_path = dir.join(format!("{stem}-code-{n}.txt"));
        n += 1;
    }
    Ok(o_path.to_string_lossy().to_string())
}
