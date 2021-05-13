use serial::prelude::*;
use std::io::{self, Read};
use std::process;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use xsens_mti::prelude::*;

// TODO
// -> Result<(), Box<dyn std::error::Error>>
// use err_derive on the crate error types, remove the unwraps
fn main() -> Result<(), io::Error> {
    let running = Arc::new(AtomicUsize::new(0));
    let r = running.clone();
    ctrlc::set_handler(move || {
        let prev = r.fetch_add(1, Ordering::SeqCst);
        if prev == 0 {
            println!("Shutting down");
        } else {
            println!("Force exit");
            process::exit(0);
        }
    })
    .expect("Error setting Ctrl-C handler");

    let mut read_buffer = vec![0_u8; 2048];
    let mut dec_buffer = vec![0_u8; 2048];
    let mut decoder = Decoder::new(&mut dec_buffer).unwrap();

    let mut port = serial::open("/dev/ttyUSB0")?;

    port.reconfigure(&|settings| {
        settings.set_baud_rate(serial::Baud115200)?;
        settings.set_char_size(serial::Bits8);
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        settings.set_flow_control(serial::FlowNone);
        Ok(())
    })?;

    port.set_timeout(Duration::from_millis(5000))?;

    while running.load(Ordering::SeqCst) == 0 {
        let bytes_read = match port.read(&mut read_buffer) {
            Ok(cnt) => cnt,
            Err(e) => {
                if matches!(e.kind(), io::ErrorKind::Interrupted) {
                    0
                } else {
                    return Err(e);
                }
            }
        };
        for byte in read_buffer[..bytes_read].iter() {
            match decoder.decode(*byte) {
                Ok(maybe_frame) => match maybe_frame {
                    Some(f) => {
                        println!("{}", f);
                        if f.message_id() == MTData2::MSG_ID {
                            println!("  MTData2");
                            let msg = MTData2::decode(&f).unwrap();
                            for (idx, pkt_result) in msg.into_iter().enumerate() {
                                let pkt = pkt_result.unwrap();
                                println!("    [{}] {}", idx, pkt);
                                let data_id = pkt.data_id();
                                match data_id.data_type() {
                                    DataType::UtcTime => {
                                        let data = UtcTime::from_be_slice(pkt.payload()).unwrap();
                                        println!("      {}", data);
                                    }
                                    DataType::PacketCounter => {
                                        let data =
                                            PacketCounter::from_be_slice(pkt.payload()).unwrap();
                                        println!("      {}", data);
                                    }
                                    DataType::SampleTimeFine => {
                                        let data =
                                            SampleTimeFine::from_be_slice(pkt.payload()).unwrap();
                                        println!("      {}", data);
                                    }
                                    DataType::SampleTimeCoarse => {
                                        let data =
                                            SampleTimeCoarse::from_be_slice(pkt.payload()).unwrap();
                                        println!("      {}", data);
                                    }
                                    DataType::EulerAngles => {
                                        if matches!(data_id.precision(), Precision::Float32) {
                                            let data =
                                                EulerAngles::<f32>::from_be_slice(pkt.payload())
                                                    .unwrap();
                                            println!("      {}", data);
                                        }
                                    }
                                    DataType::Acceleration => {
                                        if matches!(data_id.precision(), Precision::Float32) {
                                            let data =
                                                Acceleration::<f32>::from_be_slice(pkt.payload())
                                                    .unwrap();
                                            println!("      {}", data);
                                        }
                                    }
                                    DataType::RateOfTurn => {
                                        if matches!(data_id.precision(), Precision::Float32) {
                                            let data =
                                                RateOfTurn::<f32>::from_be_slice(pkt.payload())
                                                    .unwrap();
                                            println!("      {}", data);
                                        }
                                    }
                                    DataType::AltitudeEllipsoid => {
                                        if matches!(data_id.precision(), Precision::Float64) {
                                            let data = AltitudeEllipsoid::<f64>::from_be_slice(
                                                pkt.payload(),
                                            )
                                            .unwrap();
                                            println!("      {}", data);
                                        }
                                    }
                                    DataType::PositionEcef => {
                                        if matches!(data_id.precision(), Precision::Float64) {
                                            let data =
                                                PositionEcef::<f64>::from_be_slice(pkt.payload())
                                                    .unwrap();
                                            println!("      {}", data);
                                        }
                                    }
                                    DataType::LatLon => {
                                        if matches!(data_id.precision(), Precision::Float64) {
                                            let data = LatLon::<f64>::from_be_slice(pkt.payload())
                                                .unwrap();
                                            println!("      {}", data);
                                        }
                                    }
                                    DataType::VelocityXYZ => {
                                        if matches!(data_id.precision(), Precision::Float32) {
                                            let data =
                                                VelocityXYZ::<f32>::from_be_slice(pkt.payload())
                                                    .unwrap();
                                            println!("      {}", data);
                                        }
                                    }
                                    DataType::StatusWord => {
                                        let data =
                                            StatusWord::from_be_slice(pkt.payload()).unwrap();
                                        println!("      {}", data);
                                    }

                                    _ => (),
                                }
                            }
                        }
                    }
                    None => (),
                },
                Err(e) => eprintln!("Decoder error {:?}", e),
            }
        }
    }

    println!("Count: {}", decoder.count());
    println!("Invalid count: {}", decoder.invalid_count());

    Ok(())
}
