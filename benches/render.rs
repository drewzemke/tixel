use std::{fs::File, hint::black_box, io::Write};

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use tixel::{Color, HalfCellCanvas};

fn fill_checkerboard(canvas: &mut HalfCellCanvas) {
    let red = Color::new(255, 0, 0);
    let blue = Color::new(0, 0, 255);

    for y in 0..canvas.height() {
        for x in 0..canvas.width() {
            let color = if (y + x) % 2 == 0 { red } else { blue };
            canvas.set_color(x, y, color);
        }
    }
}

fn fill_solid(canvas: &mut HalfCellCanvas) {
    let color = Color::new(100, 200, 120);

    for y in 0..canvas.height() {
        for x in 0..canvas.width() {
            canvas.set_color(x, y, color);
        }
    }
}

fn bench_render_frame(c: &mut Criterion) {
    let sizes = [("40x12", 40, 12), ("80x24", 80, 24), ("160x48", 160, 48)];

    let mut group = c.benchmark_group("halfcell/checkerboard/render_frame");

    for (name, cols, rows) in sizes {
        let mut canvas = HalfCellCanvas::new(rows, cols);
        fill_checkerboard(&mut canvas);

        let output = canvas.render();

        group.throughput(Throughput::Bytes(output.len() as u64));
        group.bench_function(name, |b| {
            b.iter(|| {
                let o = canvas.render();
                black_box(o);
            })
        });
    }

    group.finish();

    let mut group = c.benchmark_group("halfcell/solid/render_frame");

    for (name, cols, rows) in sizes {
        let mut canvas = HalfCellCanvas::new(rows, cols);
        fill_solid(&mut canvas);

        let output = canvas.render();

        group.throughput(Throughput::Bytes(output.len() as u64));
        group.bench_function(name, |b| {
            b.iter(|| {
                let o = canvas.render();
                black_box(o);
            })
        });
    }

    group.finish();
}

fn bench_render_and_write(c: &mut Criterion) {
    let sizes = [("40x12", 40, 12), ("80x24", 80, 24), ("160x48", 160, 48)];

    let mut group = c.benchmark_group("halfcell/checkerboard/render_and_write");

    for (name, cols, rows) in sizes {
        let mut canvas = HalfCellCanvas::new(rows, cols);
        fill_checkerboard(&mut canvas);

        let output = canvas.render();
        let mut file = File::create("/dev/null").unwrap();

        group.throughput(Throughput::Bytes(output.len() as u64));
        group.bench_function(name, |b| {
            b.iter(|| {
                let o = canvas.render();
                file.write_all(o.as_bytes()).unwrap();
            })
        });
    }

    group.finish();

    let mut group = c.benchmark_group("halfcell/solid/render_and_write");

    for (name, cols, rows) in sizes {
        let mut canvas = HalfCellCanvas::new(rows, cols);
        fill_solid(&mut canvas);

        let output = canvas.render();
        let mut file = File::create("/dev/null").unwrap();

        group.throughput(Throughput::Bytes(output.len() as u64));
        group.bench_function(name, |b| {
            b.iter(|| {
                let o = canvas.render();
                file.write_all(o.as_bytes()).unwrap();
            })
        });
    }

    group.finish();
}

criterion_group!(benches, bench_render_frame, bench_render_and_write);
criterion_main!(benches);
