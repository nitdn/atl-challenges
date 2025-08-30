use super::*;
const WEIGHTS: [usize; 8] = [2, 5, 1, 8, 4, 3, 6, 7];

#[test]
#[should_panic]
fn test_invalid_queue() {
    let _: Packet = Packet::new(132, 42, 1).unwrap();
}

#[test]
fn test_flow_order() {
    let mut scheduler = Scheduler::new(WEIGHTS);
    let flow_1: Vec<_> = (0..10)
        .map(|payload| Packet::new(payload, 0, 1).unwrap())
        .collect();
    let flow_2: Vec<_> = (0..4)
        .map(|payload| Packet::new(payload, 7, 2).unwrap())
        .collect();
    let flow_3: Vec<_> = (0..8)
        .map(|payload| Packet::new(payload, 0, 3).unwrap())
        .collect();
    scheduler.enqueue(&flow_1);
    scheduler.enqueue(&flow_2);
    scheduler.enqueue(&flow_3);
    assert_eq!(
        flow_1,
        scheduler
            .clone()
            .into_iter()
            .filter(|packet| packet.flow == 1)
            .collect::<Vec<Packet>>()
    );
    assert_eq!(
        flow_2.iter().collect::<Vec<&Packet>>(),
        scheduler
            .iter()
            .filter(|packet| packet.flow == 2)
            .collect::<Vec<&Packet>>()
    );
    assert_eq!(
        flow_3.iter().collect::<Vec<&Packet>>(),
        scheduler
            .iter()
            .filter(|packet| packet.flow == 3)
            .collect::<Vec<&Packet>>()
    );
    assert_eq!(
        scheduler.iter().collect::<Vec::<_>>(),
        scheduler
            .clone()
            .into_iter()
            .collect::<Vec::<_>>()
            .iter()
            .collect::<Vec<_>>(),
    )
}

#[test]
fn test_weighting() {
    let mut scheduler = Scheduler::new(WEIGHTS);
    for queue in 0..3 {
        let packets: Vec<Packet> = (0..5)
            .map(|flow| Packet::new(0, queue, flow).unwrap())
            .collect();
        scheduler.enqueue(&packets);
    }
    for packet in scheduler.clone().iter().take(WEIGHTS[0]) {
        assert_eq!(0, packet.class);
    }
    for packet in scheduler.iter().skip(WEIGHTS[0]).take(WEIGHTS[1]) {
        assert_eq!(1, packet.class);
    }
    for packet in scheduler
        .iter()
        .skip(WEIGHTS[0])
        .skip(WEIGHTS[1])
        .take(WEIGHTS[2])
    {
        assert_eq!(2, packet.class);
    }
}

#[test]
fn test_class_order() {
    let desired_order = [
        Packet::new(0, 0, 0).unwrap(),
        Packet::new(0, 1, 0).unwrap(),
        Packet::new(0, 2, 0).unwrap(),
        Packet::new(0, 3, 0).unwrap(),
        Packet::new(0, 4, 0).unwrap(),
        Packet::new(0, 5, 0).unwrap(),
        Packet::new(0, 6, 0).unwrap(),
        Packet::new(0, 7, 0).unwrap(),
    ];
    let input_order = [
        Packet::new(0, 3, 0).unwrap(),
        Packet::new(0, 0, 0).unwrap(),
        Packet::new(0, 5, 0).unwrap(),
        Packet::new(0, 2, 0).unwrap(),
        Packet::new(0, 7, 0).unwrap(),
        Packet::new(0, 4, 0).unwrap(),
        Packet::new(0, 1, 0).unwrap(),
        Packet::new(0, 6, 0).unwrap(),
    ];
    let scrambled_order = [
        Packet::new(0, 7, 0).unwrap(),
        Packet::new(0, 3, 0).unwrap(),
        Packet::new(0, 2, 0).unwrap(),
        Packet::new(0, 0, 0).unwrap(),
        Packet::new(0, 4, 0).unwrap(),
        Packet::new(0, 1, 0).unwrap(),
        Packet::new(0, 6, 0).unwrap(),
        Packet::new(0, 5, 0).unwrap(),
    ];
    let mut scheduler = Scheduler::default();
    scheduler.enqueue(&input_order);
    let scheduled_packets: Vec<_> = scheduler.iter().collect();
    assert_eq!(desired_order.iter().collect::<Vec<_>>(), scheduled_packets);
    assert_ne!(
        scrambled_order.iter().collect::<Vec<_>>(),
        scheduled_packets
    );
}
