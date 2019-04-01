use rkapi::tag::*;
use criterion::*;
use serde_json;

fn serialize_large(clients: &[(&str, &str)]) -> String {
    let upserts = clients.iter().map(|(name, ip)| {
        let name = name.to_string();
        let ip   = ip.to_string();
        large((name, ip))
    }).collect::<Vec<_>>();
    serde_json::to_string(&upserts).unwrap()
}

fn serialize_small(clients: &[(&str, &str)]) -> String {
    let upserts = clients.iter().map(|(name, ip)| {
        let name = name.to_string();
        let ip   = ip.to_string();
        small((name, ip))
    }).collect::<Vec<_>>();
    serde_json::to_string(&upserts).unwrap()
}

fn small((name, ip): (String, String)) -> Upsert {
    let addr = Some((ip,));
    Upsert::Small(Small{
        value:    name,
        criteria: (Rule{addr, ..Default::default()},)
    })
}

fn large((name, ip): (String, String)) -> Upsert {
    let addr = vec![ip];
    Upsert::Large(Large{
        value:    name,
        criteria: vec![Rules{addr, ..Default::default()}],
    })
}

fn tag_benchmark(c: &mut Criterion) {
    let clients = vec![
        ("alice", "10.0.0.16"),
        ("bob",   "10.0.0.32"),
    ];

    let clients0 = clients.clone();
    c.bench_function("serialize large", move |b| {
        b.iter(|| serialize_large(&clients0))
    });

    let clients1 = clients.clone();
    c.bench_function("serialize small", move |b| {
        b.iter(|| serialize_small(&clients1))
    });
}

criterion_group!(benches, tag_benchmark);
criterion_main!(benches);
