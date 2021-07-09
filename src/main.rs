#![windows_subsystem = "windows"]
use fltk::{app::*, button::*, dialog::*, frame::*, group::*, input::*, text::*, window::*};
use rand::Rng;

#[derive(Clone, Debug)]
// Define a struct for the form fields
struct Parameters {
    data_a: TextEditor,
    data_b: TextEditor,
    output: TextDisplay,
    paired_data: CheckButton,
    one_tailed: RadioRoundButton,
    two_tailed: RadioRoundButton,
    cinterval: FloatInput,
    iterations: IntInput,
}

#[derive(Clone, Debug)]
// Define a struct for our dmeans and dsds
struct Sdmeanresults {
    mean: f64,
    sd: f64,
}

fn main() {
    let app = App::default();

    // Main Window
    let mut wind = Window::new(
        100,
        100,
        737,
        530,
        "Bootstrap Mean Difference & Spearman Calculator v2.70",
    );

    // Fill the form structure
    let mut parameters = Parameters {
        data_a: TextEditor::new(16, 30, 204, 404, ""),
        data_b: TextEditor::new(247, 30, 204, 404, ""),
        paired_data: CheckButton::new(556, 26, 105, 21, "Paired or Corr Data"),
        one_tailed: RadioRoundButton::new(558, 54, 99, 21, "One Tailed"),
        two_tailed: RadioRoundButton::new(558, 81, 99, 21, "Two Tailed"),
        cinterval: FloatInput::new(558, 119, 54, 22, "CL"),
        iterations: IntInput::new(558, 148, 54, 22, "Iterations"),
        output: TextDisplay::new(480, 200, 230, 300, ""),
    };

    // Text buffers for our inputs and output
    let buf_a = TextBuffer::default();
    let buf_b = TextBuffer::default();
    let buf_out = TextBuffer::default();

    // Data Labels for Main Input windows
    Frame::new(16, 10, 51, 17, "Data A");
    Frame::new(255, 10, 51, 17, "Data B");
    Frame::new(610, 148, 20, 22, "K");
    Frame::new(610, 119, 20, 22, "%");

    // Format and initialize our main input windows
    parameters.data_a.set_scrollbar_size(15);
    parameters.data_b.set_scrollbar_size(15);
    parameters.data_a.set_cursor_style(TextCursor::Simple);
    parameters.data_b.set_cursor_style(TextCursor::Simple);
    parameters.data_a.set_buffer(Some(buf_a));
    parameters.data_b.set_buffer(Some(buf_b));
    parameters.data_a.set_tab_nav(true);
    parameters.data_b.set_tab_nav(true);

    // Set output buffer
    parameters.output.set_buffer(Some(buf_out));

    // Group for radio buttons
    let mut group_tailed = Group::new(555, 50, 100, 50, "");
    group_tailed.add(&parameters.one_tailed);
    group_tailed.add(&parameters.two_tailed);
    group_tailed.end();

    // Set intial values for the form
    parameters.two_tailed.toggle(true);
    parameters.cinterval.set_value("95");
    parameters.iterations.set_value("10");

    // Clone the parameters to use for the clear function
    let mut p2 = parameters.clone();

    // Calculate button
    let mut calculate_button = Button::new(130, 450, 200, 57, "Calculate");
    calculate_button.set_callback(move || calculate(&mut parameters));

    // clear button
    let mut clear_button = Button::new(350, 450, 100, 57, "Clear");
    clear_button.set_callback(move || clear(&mut p2));

    // Show the window
    wind.end();
    wind.show();

    // Enter main loop
    app.run().unwrap();
}

fn clear(p: &mut Parameters) {
    p.output.buffer().unwrap().set_text("");
    p.data_a.buffer().unwrap().set_text("");
    p.data_b.buffer().unwrap().set_text("");
}

// Handle Calculate button
fn calculate(p: &mut Parameters) {
    let sdmeanresults: Sdmeanresults;

    // Output String
    let mut out: String = String::from("");

    // Get the CSV data out of the two data fields
    let a_v: Vec<f64> = csv_split(&p.data_a.buffer().unwrap().text());
    let b_v: Vec<f64> = csv_split(&p.data_b.buffer().unwrap().text());

    if a_v.len() < 1 {
        alert(368, 265, "Data A must have a least one value");
        return;
    }

    if b_v.len() < 1 {
        alert(368, 265, "Data B must have a least one value");
        return;
    }

    // Get our iteration count
    let iterations: i32 = match p.iterations.value().parse::<i32>() {
        Ok(v) => v * 1000,
        Err(_) => {
            alert(368, 265, "Iteration Count Error");
            return;
        }
    };
    if iterations < 1000 || iterations > 9999000 {
        alert(368, 265, "Iteration Count Error");
        return;
    };

    // Get our Confidence Level
    let confidence: f64 = match p.cinterval.value().parse::<f64>() {
        Ok(v) => v,
        Err(_) => {
            alert(368, 265, "Confidence Level Error");
            return;
        }
    };
    if confidence < 0.0 || confidence > 100.0 {
        alert(368, 265, "Confidence Level Error");
        return;
    };

    let clevel: f64 = (100.0 - confidence) / 100.0;

    // Check for paired or unpaired data
    if p.paired_data.is_checked() {
        // For paired data make sure both columns have the same number of elements
        if a_v.len() != b_v.len() {
            alert(368, 265, "Data Fields Must Have Same Count for Paired Data");
            return;
        }

        sdmeanresults = paired_data(&a_v, &b_v, iterations);
    } else {
        sdmeanresults = unpaired_data(&a_v, &b_v, iterations);
    };

    // Calculate stats for the data
    let mean_a = mean(&a_v);
    let mean_b = mean(&b_v);
    let mean_d = mean_b - mean_a;
    let sd_a = sd_sample(&a_v, &mean_a);
    let sd_b = sd_sample(&b_v, &mean_b);
    let sd_d = sd_b - sd_a;
    let sd_pooled = ((sd_a * sd_a + sd_b * sd_b) / 2.0).sqrt();
    let d = mean_d / sd_pooled;
    let sk_a = skewness(&a_v, &mean_a, &sd_a);
    let sk_b = skewness(&b_v, &mean_b, &sd_b);
    let kt_a = kurtosis(&a_v, &mean_a, &sd_a);
    let kt_b = kurtosis(&b_v, &mean_b, &sd_b);

    out.push_str(&format!("Count A: \t{}\n", a_v.len()));
    out.push_str(&format!("Count B: \t{}\n", b_v.len()));
    out.push_str("\n************************************\n");

    // Handle one or two tailed data Mean
    if p.two_tailed.is_toggled() {
        // Two Tailed
        let z = z_from_cl(1.0 - clevel);
        let u = mean_d + z * sdmeanresults.mean;
        let l = mean_d - z * sdmeanresults.mean;
        let pv = p_from_ci(l, u, mean_d, 1.0 - clevel);

        out.push_str(&format!(
            "Mean A: \t{}\n",
            &science_pretty_format(mean_a, 6)
        ));
        out.push_str(&format!(
            "Mean B: \t{}\n",
            &science_pretty_format(mean_b, 6)
        ));
        out.push_str("\n");
        out.push_str(&format!("Low Diff: \t{}\n", &science_pretty_format(l, 6)));
        out.push_str(&format!(
            "Mean Diff: \t{}\n",
            &science_pretty_format(mean_d, 6)
        ));
        out.push_str(&format!("High Diff: \t{}\n", &science_pretty_format(u, 6)));
        out.push_str(&format!("\np-Value: \t{}\n", &science_pretty_format(pv, 3)));

        if l <= 0.0 && u >= 0.0 {
            out.push_str("H0 = True \tA ≈ B\n");
        } else {
            if mean_a > mean_b {
                out.push_str("H0 = False \tA > B\n");
            } else {
                out.push_str("H0 = False \tA < B\n");
            }
        };
    } else {
        // One Tailed
        let z = z_from_cl(1.0 - clevel * 2.0);
        let u = mean_d + z * sdmeanresults.mean;
        let l = mean_d - z * sdmeanresults.mean;
        let pv = p_from_ci(l, u, mean_d, 1.0 - clevel);

        out.push_str(&format!(
            "Mean A: \t{}\n",
            &science_pretty_format(mean_a, 6)
        ));
        out.push_str(&format!(
            "Mean B: \t{}\n",
            &science_pretty_format(mean_b, 6)
        ));
        out.push_str("\n");

        if mean_a > mean_b {
            out.push_str(&format!("Low Diff: \t{}\n", &science_pretty_format(l, 6)));
            out.push_str(&format!(
                "Mean Diff: \t{}\n",
                &science_pretty_format(mean_d, 6)
            ));
            out.push_str(&format!("\np-Value: \t{}\n", &science_pretty_format(pv, 3)));
            if u >= 0.0 {
                out.push_str("H0 = True \tA ≈ B\n");
            } else {
                out.push_str("H0 = False \tA > B\n");
            }
        } else {
            out.push_str(&format!(
                "Mean Diff: \t{}\n",
                &science_pretty_format(mean_d, 6)
            ));
            out.push_str(&format!("High Diff: \t{}\n", &science_pretty_format(u, 6)));
            out.push_str(&format!("\np-Value: \t{}\n", &science_pretty_format(pv, 3)));
            if l <= 0.0 {
                out.push_str("H0 = True \tA ≈ B\n");
            } else {
                out.push_str("H0 = False \tA < B\n");
            }
        }
    }

    out.push_str("\n************************************\n");

    // Handle one or two tailed data SD
    if p.two_tailed.is_toggled() {
        // Two Tailed
        let z = z_from_cl(1.0 - clevel);
        let u = sd_d + z * sdmeanresults.sd;
        let l = sd_d - z * sdmeanresults.sd;
        let pv = p_from_ci(l, u, sd_d, 1.0 - clevel);

        out.push_str(&format!("SD A:    \t{}\n", &science_pretty_format(sd_a, 6)));
        out.push_str(&format!("SD B:    \t{}\n", &science_pretty_format(sd_b, 3)));
        out.push_str("\n");
        out.push_str(&format!("Low Diff: \t{}\n", &science_pretty_format(l, 6)));
        out.push_str(&format!("SD Diff: \t{}\n", &science_pretty_format(sd_d, 6)));
        out.push_str(&format!("High Diff: \t{}\n", &science_pretty_format(u, 6)));
        out.push_str(&format!("\np-Value: \t{}\n", &science_pretty_format(pv, 3)));

        if l <= 0.0 && u >= 0.0 {
            out.push_str("H0 = True \tA ≈ B\n");
        } else {
            if sd_a > sd_b {
                out.push_str("H0 = False \tA > B\n");
            } else {
                out.push_str("H0 = False \tA < B\n");
            }
        };
    } else {
        // One Tailed
        let z = z_from_cl(1.0 - clevel * 2.0);
        let u = sd_d + z * sdmeanresults.sd;
        let l = sd_d - z * sdmeanresults.sd;
        let pv = p_from_ci(l, u, sd_d, 1.0 - clevel);

        out.push_str(&format!("SD A:    \t{}\n", &science_pretty_format(sd_a, 6)));
        out.push_str(&format!("SD B:    \t{}\n", &science_pretty_format(sd_b, 6)));
        out.push_str("\n");

        if sd_a > sd_b {
            out.push_str(&format!("Low Diff: \t{}\n", &science_pretty_format(l, 6)));
            out.push_str(&format!("SD Diff: \t{}\n", &science_pretty_format(sd_d, 6)));
            out.push_str(&format!("\np-Value: \t{}\n", &science_pretty_format(pv, 3)));
            if u >= 0.0 {
                out.push_str("H0 = True \tA ≈ B\n");
            } else {
                out.push_str("H0 = False \tA > B\n");
            }
        } else {
            out.push_str(&format!("SD Diff: \t{}\n", &science_pretty_format(sd_d, 6)));
            out.push_str(&format!("High Diff: \t{}\n", &science_pretty_format(u, 6)));
            out.push_str(&format!("\np-Value: \t{}\n", &science_pretty_format(pv, 3)));
            if l <= 0.0 {
                out.push_str("H0 = True \tA ≈ B\n");
            } else {
                out.push_str("H0 = False \tA < B\n");
            }
        }
    }

    out.push_str("\n************************************\n");

    out.push_str(&format!("Cohen's d: \t{}\n", &science_pretty_format(d, 2)));

    out.push_str("\n************************************\n");

    out.push_str(&format!("SE A:    \t{}\n", &science_pretty_format(sd_a / (a_v.len() as f64).sqrt(), 6)));
    out.push_str(&format!("SE B:    \t{}\n", &science_pretty_format(sd_b / (b_v.len() as f64).sqrt(), 6)));

    out.push_str("\n************************************\n");

    out.push_str(&format!("Skewness A:    \t{}\n", &science_pretty_format(sk_a, 3)));
    out.push_str(&format!("Skewness B:    \t{}\n", &science_pretty_format(sk_b, 3)));
    out.push_str(&format!("\nKurtosis A:    \t{}\n", &science_pretty_format(kt_a, 3)));
    out.push_str(&format!("Kurtosis B:    \t{}\n", &science_pretty_format(kt_b, 3)));

    out.push_str("\n************************************\n");

    // Check for paired correlation data
    if p.paired_data.is_checked() {
        // Perform correlation calculations
        if a_v.len() > 1 {
            let r = r_value(rankify(&a_v), rankify(&b_v));

            out.push_str(&format!("Spearman's ρ: \t{}\n", &science_pretty_format(r, 2)));

            let cstring = match r {
                r if r == 0.0 => "None",
                r if r == 1.0 => "Perfect Pos",
                r if r == -1.0 => "Perfect Neg",
                r if r > 0.0 && r < 0.3 => "Weak Pos",
                r if r >= 0.3 && r < 0.7 => "Moderate Pos",
                r if r >= 0.7 && r < 1.0 => "Strong Pos",
                r if r < 0.0 && r > -0.3 => "Weak Neg",
                r if r <= -0.3 && r > -0.7 => "Moderate Neg",
                r if r <= -0.7 && r > -1.00 => "Strong Neg",
                _ => "",
            };

            out.push_str(&format!("Corr:      \t{}\n", &cstring));

            let dof = a_v.len() as f64 - 2.0;
            let tr = r / ((1.0 - r * r) / dof).sqrt();
            let pr = p_from_t(tr, dof);

            out.push_str(&format!("\np-Value: \t{}\n", &science_pretty_format(pr, 3)));

            if pr <= clevel {
                out.push_str("Sig:       \tSignificant\n");
            } else {
                out.push_str("Sig:       \tNot Significant\n");
            }
        }
    }

    p.output.buffer().unwrap().set_text(&out);
}

// Paired data
fn paired_data(a_v: &Vec<f64>, b_v: &Vec<f64>, iterations: i32) -> Sdmeanresults {
    let mut dmeans: Vec<f64> = Vec::new();
    let mut dsds: Vec<f64> = Vec::new();
    let mut tmp: Vec<f64> = Vec::new();

    let mut cvalues: Vec<f64> = Vec::new();

    let l = a_v.len();

    for i in 0..l {
        cvalues.push(b_v[i as usize] - a_v[i as usize]);
    }

    for _i in 0..iterations {
        tmp.clear();
        for _j in 0..l {
            tmp.push(cvalues[rand::thread_rng().gen_range(0..l)]);
        }
        let m: f64 = mean(&tmp);
        dmeans.push(m);
        dsds.push(sd_sample(&tmp, &m));
    }

    Sdmeanresults {
        mean: sd_sample(&dmeans, &mean(&dmeans)),
        sd: sd_sample(&dsds, &mean(&dsds)),
    }
}

// Unpaired data
fn unpaired_data(a_v: &Vec<f64>, b_v: &Vec<f64>, iterations: i32) -> Sdmeanresults {
    let mut dmeans: Vec<f64> = Vec::new();
    let mut ameans: Vec<f64> = Vec::new();
    let mut bmeans: Vec<f64> = Vec::new();

    let mut dsds: Vec<f64> = Vec::new();
    let mut asds: Vec<f64> = Vec::new();
    let mut bsds: Vec<f64> = Vec::new();

    let mut tmp: Vec<f64> = Vec::new();

    let avl = a_v.len();
    let bvl = b_v.len();

    for _i in 0..iterations {
        tmp.clear();
        for _j in 0..avl {
            tmp.push(a_v[rand::thread_rng().gen_range(0..avl)]);
        }
        let m: f64 = mean(&tmp);
        ameans.push(m);
        asds.push(sd_sample(&tmp, &m));
    }

    for _i in 0..iterations {
        tmp.clear();
        for _j in 0..bvl {
            tmp.push(b_v[rand::thread_rng().gen_range(0..bvl)]);
        }
        let m: f64 = mean(&tmp);
        bmeans.push(m);
        bsds.push(sd_sample(&tmp, &m));
    }

    for i in 0..iterations {
        dmeans.push(bmeans[i as usize] - ameans[i as usize]);
        dsds.push(bsds[i as usize] - asds[i as usize])
    }

    Sdmeanresults {
        mean: sd_sample(&dmeans, &mean(&dmeans)),
        sd: sd_sample(&dsds, &mean(&dsds)),
    }
}

// Convert CSV from the main windows to arrays of floats, also clean up stray whitespace
fn csv_split(inp: &String) -> Vec<f64> {
    let mut values: Vec<f64> = Vec::new();

    let clean_inp: String = inp
        .replace("\n", ",")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();

    let fields = clean_inp.split(",");

    for f in fields {
        match f.parse::<f64>() {
            Ok(v) => values.push(v),
            Err(_) => continue,
        };
    }

    values
}

// Calculate mean
fn mean(vec: &Vec<f64>) -> f64 {
    let sum: f64 = Iterator::sum(vec.iter());
    sum / vec.len() as f64
}

// Calculate SD of a sample
fn sd_sample(x: &Vec<f64>, mean: &f64) -> f64 {
    let mut sd: f64 = 0.0;
    let size: usize = x.len();

    for i in 0..size {
        sd += (x[i] - mean).powf(2.0);
    }
    (sd / (size - 1) as f64).sqrt()
}

// Calculate Skewness
fn skewness(vec: &Vec<f64>, mean: &f64, sd: &f64) -> f64 {
    let sz:f64 = vec.len() as f64;
    let mut tmpsum: f64 = 0.0;
    let sdp = sd.powf(3.0);

    for v in &mut vec.iter() {
        tmpsum += (v - mean).powf(3.0) / sdp;
    }
    
    (sz / ((sz - 1.0) * (sz - 2.0))) * tmpsum
}

// Calculate Kurtosis
fn kurtosis(vec: &Vec<f64>, mean: &f64, sd: &f64) -> f64 {
    let sz:f64 = vec.len() as f64;
    let mut tmpsum: f64 = 0.0;
    let sdp = sd.powf(4.0);

    for v in &mut vec.iter() {
        tmpsum += (v - mean).powf(4.0) / sdp;
    }

    (((sz * (sz + 1.0)) / ((sz - 1.0) * (sz - 2.0) * (sz - 3.0))) * tmpsum) - ((3.0 * (sz - 1.0) * (sz - 1.0)) / ((sz - 2.0) * (sz - 3.0)))
}


// Rankify
fn rankify(x: &Vec<f64>) -> Vec<f64> {
    let n = x.len();
    let mut rank: Vec<f64> = Vec::new();

    for i in 0..n {
        let mut r = 1;
        let mut s = 1;

        for j in 0..i {
            if x[j] < x[i] {
                r += 1;
            }
            if x[j] == x[i] {
                s += 1;
            }
        }

        for j in (i + 1)..n {
            if x[j] < x[i] {
                r += 1;
            }
            if x[j] == x[i] {
                s += 1;
            }
        }

        rank.push(r as f64 + (s as f64 - 1.0) * 0.5);
    }
    rank
}

// Calculate R Correlation
fn r_value(x: Vec<f64>, y: Vec<f64>) -> f64 {
    let mut xmx_sum: f64 = 0.0;
    let mut ymy_sum: f64 = 0.0;
    let mut xmx_ymy_sum: f64 = 0.0;

    let mx = mean(&x);
    let my = mean(&y);

    for i in 0..x.len() {
        xmx_sum += (x[i] - mx) * (x[i] - mx);
        ymy_sum += (y[i] - my) * (y[i] - my);
        xmx_ymy_sum += (x[i] - mx) * (y[i] - my);
    }

    xmx_ymy_sum / (xmx_sum * ymy_sum).sqrt()
}

// Calculate Log Gamma
fn l_gamma(x: f64) -> f64 {
    let coef: [f64; 6] = [
        76.18009172947146,
        -86.50532032941677,
        24.01409824083091,
        -1.231739572450155,
        0.1208650973866179E-2,
        -0.5395239384953E-5,
    ];
    let logsqrttwopi: f64 = 0.91893853320467274178;
    let y: f64 = x + 5.5;
    let mut denom: f64 = x + 1.0;
    let mut series: f64 = 1.000000000190015;

    for i in 0..6 {
        series += coef[i] / denom;
        denom += 1.0;
    }

    logsqrttwopi + (x + 0.5) * y.ln() - y + (series / x).ln()
}

// Calculate P Value from T statistic
fn p_from_t(ws: f64, dof: f64) -> f64 {
    let a: f64 = dof / 2.0;
    let mut value = dof / (ws * ws + dof);

    if value.is_infinite() || value.is_nan() {
        return 1.0;
    }

    let beta = l_gamma(a) + 0.57236494292470009 - l_gamma(a + 0.5);
    let acu = 0.1E-14;
    let mut ai: f64;
    let mut cx: f64;
    let mut ns: i32;
    let mut psq: f64;
    let mut rx: f64;
    let mut temp: f64;
    let mut term: f64;
    let xx: f64;
    let qq: f64;
    let pp: f64;
    let indx: i32;

    if value < 0.0 || 1.0 < value {
        return value;
    }

    if value == 0.0 || value == 1.0 {
        return value;
    }

    psq = a + 0.5;
    cx = 1.0 - value;

    if a < psq * value {
        xx = cx;
        cx = value;
        pp = 0.5;
        qq = a;
        indx = 1;
    } else {
        xx = value;
        pp = a;
        qq = 0.5;
        indx = 0;
    }

    term = 1.0;
    ai = 1.0;
    value = 1.0;
    ns = (qq + cx * psq) as i32;
    rx = xx / cx;
    temp = qq - ai;

    if ns == 0 {
        rx = xx;
    }

    loop {
        term = term * temp * rx / (pp + ai);
        value += term;
        temp = term.abs();

        if temp <= acu && temp <= acu * value {
            value = value * (pp * xx.ln() + (qq - 1.0) * cx.ln() - beta).exp() / pp;

            if indx != 0 {
                value = 1.0 - value;
            }

            break;
        }

        ai += 1.0;
        ns -= 1;

        if 0 <= ns {
            temp = qq - ai;

            if ns == 0 {
                rx = xx;
            }
        } else {
            temp = psq;
            psq += 1.0;
        }
    }

    value
}

// Calculate P-Value from CI
fn p_from_ci(l: f64, u: f64, m: f64, cl: f64) -> f64 {
    let s: f64 = erf_inv(cl) * 1.414213562373095;
    let se: f64 = (u - l) / (2.0 * s);
    let z: f64 = m / se;

    (1.0 - p_from_z(z.abs())) * 2.0
}

// Calculate inverse ERF
fn erf_inv(x: f64) -> f64 {
    let mut w: f64;
    let mut p: f64;

    w = -((1.0 - x) * (1.0 + x)).ln();

    if w < 5.000000 {
        w = w - 2.500000;
        p = 2.81022636e-08;
        p = 3.43273939e-07 + p * w;
        p = -3.5233877e-06 + p * w;
        p = -4.39150654e-06 + p * w;
        p = 0.00021858087 + p * w;
        p = -0.00125372503 + p * w;
        p = -0.00417768164 + p * w;
        p = 0.246640727 + p * w;
        p = 1.50140941 + p * w;
    } else {
        w = w.sqrt() - 3.000000;
        p = -0.000200214257;
        p = 0.000100950558 + p * w;
        p = 0.00134934322 + p * w;
        p = -0.00367342844 + p * w;
        p = 0.00573950773 + p * w;
        p = -0.0076224613 + p * w;
        p = 0.00943887047 + p * w;
        p = 1.00167406 + p * w;
        p = 2.83297682 + p * w;
    }

    p * x
}

// Calculate Z from Confidence Level
fn z_from_cl(cl: f64) -> f64 {
    erf_inv(cl) * 1.414213562373095
}

// Calculate P from Z
fn p_from_z(z: f64) -> f64 {
    let mut y: f64;
    let x: f64;
    let w: f64;

    if z == 0.0 {
        x = 0.0;
    } else {
        y = 0.5 * z.abs();

        if y >= 3.0 {
            x = 1.0;
        } else {
            if y < 1.0 {
                w = y * y;
                x = ((((((((0.000124818987 * w - 0.001075204047) * w + 0.005198775019) * w
                    - 0.019198292004)
                    * w
                    + 0.059054035642)
                    * w
                    - 0.151968751364)
                    * w
                    + 0.319152932694)
                    * w
                    - 0.531923007300)
                    * w
                    + 0.797884560593)
                    * y
                    * 2.0;
            } else {
                y -= 2.0;
                x = (((((((((((((-0.000045255659 * y + 0.000152529290) * y
                    - 0.000019538132)
                    * y
                    - 0.000676904986)
                    * y
                    + 0.001390604284)
                    * y
                    - 0.000794620820)
                    * y
                    - 0.002034254874)
                    * y
                    + 0.006549791214)
                    * y
                    - 0.010557625006)
                    * y
                    + 0.011630447319)
                    * y
                    - 0.009279453341)
                    * y
                    + 0.005353579108)
                    * y
                    - 0.002141268741)
                    * y
                    + 0.000535310849)
                    * y
                    + 0.999936657524;
            }
        }
    }

    if z > 0.0 {
        return (x + 1.0) * 0.5;
    } else {
        return (1.0 - x) * 0.5;
    }
}

// Pretty Format Scientific Numbers
fn science_pretty_format(value: f64, digits: usize) -> String {
    if value.abs() == 0.0 {
        return "0".to_string();
    }
    if value.abs() >= 10000.0
        || value.abs() < 0.001
    {
        return format!("{:.*e}", digits, value).to_string();
    }
    format!("{:.*}", digits, value)
        .trim_end_matches(|c| c == '0')
        .trim_end_matches(|c| c == '.')
        .to_string()
}
