use std::io::Write;

pub(crate) fn analyze(
    path: &str,
    t_len: u64,
    cost: f64,
    layout: &Option<crate::layout_map::LayoutMap>,
) -> crate::DynResult<Vec<u8>> {
    let mut buf = Vec::with_capacity(1024);
    writeln!(buf)?; // 空行分隔
    writeln!(buf, "----统计数据----")?;
    writeln!(buf, "总字数\t{t_len}")?;

    let Some(layout) = layout else {
        let mut c_len = 0u64;
        crate::utils::for_each_chunk(path, |s| Ok(c_len += s.chars().count() as u64))?;
        writeln!(buf, "总码数\t{c_len}")?;
        writeln!(buf, "总开销\t{cost}")?;
        writeln!(buf, "字均码长\t{}", c_len as f64 / t_len as f64)?;
        writeln!(buf, "字均开销\t{}", cost / t_len as f64)?;
        writeln!(buf, "码均开销\t{}", cost / c_len as f64)?;
        writeln!(buf, "键盘布局未知，不分析使用率")?;
        return Ok(buf);
    };

    let mut c_len = 0u64;
    let mut finger_cnt = [0u64; 9]; // 各手指计数
    let mut row_cnt = [0u64; 5]; // 各排计数
    let mut repeat_len = 1; // 同键连击长度
    let mut repeat_cnt = [0u64; 4]; // 同键2-5+连击计数
    let mut leap_cnt = [0u64; 3]; // 同指跨1-3排计数
    let mut switch_cnt = 0u64; // 左右手互击计数
    let (mut c0, mut f0, mut r0) = ('\0', 0, 0u8);
    let not_thumb_nor_space = |f: u8, r: u8| f > 0 && f < 9 && r > 0 && r < 5;
    crate::utils::for_each_chunk(path, |s| {
        Ok(for c1 in s.chars() {
            c_len += 1;
            let (f1, r1) = layout.get(c1);
            if f1 > 0 {
                finger_cnt[f1 as usize - 1] += 1;
            }
            if r1 > 0 {
                row_cnt[r1 as usize - 1] += 1;
            }
            if c0 == c1 {
                repeat_len += 1;
            } else {
                if repeat_len > 1 {
                    repeat_cnt[3.min(repeat_len as usize - 2)] += 1;
                    repeat_len = 1;
                }
                if not_thumb_nor_space(f0, r0) && not_thumb_nor_space(f1, r1) {
                    if f0 == f1 {
                        let row_diff = r0.abs_diff(r1);
                        if row_diff > 0 {
                            leap_cnt[row_diff as usize - 1] += 1;
                        }
                    } else if (f0 < 5) != (f1 < 5) {
                        switch_cnt += 1;
                    }
                }
            }
            (c0, f0, r0) = (c1, f1, r1);
        })
    })?;

    if repeat_len > 1 {
        repeat_cnt[3.min(repeat_len as usize - 2)] += 1;
    }
    let left_sum: u64 = finger_cnt[..4].iter().sum();
    let right_sum: u64 = finger_cnt[4..8].iter().sum();
    let bias_ratio = match left_sum + right_sum {
        0 => 0.0,
        sum => 100.0 * left_sum.abs_diff(right_sum) as f64 / sum as f64,
    };

    let add_info = |buf: &mut Vec<u8>, name: &str, cnt: u64, window_size: u64| {
        if c_len < window_size {
            return writeln!(buf, "{name}\t{cnt}");
        }
        let ratio = 100.0 * cnt as f64 / (c_len - window_size + 1) as f64;
        writeln!(buf, "{name}\t{cnt}\t{ratio}%")
    };
    writeln!(buf, "总码数\t{c_len}")?;
    writeln!(buf, "总开销\t{cost}")?;
    writeln!(buf, "字均码长\t{}", c_len as f64 / t_len as f64)?;
    writeln!(buf, "字均开销\t{}", cost / t_len as f64)?;
    writeln!(buf, "码均开销\t{}", cost / c_len as f64)?;
    writeln!(buf, "偏倚\t{bias_ratio}%")?;
    add_info(&mut buf, "互击", switch_cnt, 2)?;
    add_info(&mut buf, "拇指", finger_cnt[8], 1)?;
    writeln!(buf, "------左手------")?;
    add_info(&mut buf, "总计", left_sum, 1)?;
    let names = ["小指", "无名", "中指", "食指"];
    for i in 0..4 {
        add_info(&mut buf, names[i], finger_cnt[i], 1)?;
    }
    writeln!(buf, "------右手------")?;
    add_info(&mut buf, "总计", right_sum, 1)?;
    for i in 0..4 {
        add_info(&mut buf, names[3 - i], finger_cnt[i + 4], 1)?;
    }
    writeln!(buf, "-------排-------")?;
    let names = ["数字", "上排", "中排", "下排", "空格"];
    for i in 0..5 {
        add_info(&mut buf, names[i], row_cnt[i], 1)?;
    }
    writeln!(buf, "----同键连击----")?;
    for i in 2..=4 {
        add_info(&mut buf, &format!("{i}连"), repeat_cnt[i - 2], i as u64)?;
    }
    writeln!(buf, "5+连\t{}", repeat_cnt[3])?;
    writeln!(buf, "----同指跨排----")?;
    for i in 1..=3 {
        add_info(&mut buf, &format!("{i}排"), leap_cnt[i - 1], 2)?;
    }

    Ok(buf)
}
