/*
# turn on
send([0x11, 0x11, 0x14, 0x39, 0x38, 0x30, 0x39])
receive(7)

# get status
send([0x34])
receive(16)

# set input 1 (3.5mm 6 channel)
send([0x09, 0x02, 0x14, 0x08])
receive(4)

send([0x09])
receive(1)
send([0x09])
receive(1)
*/

use serialport::{DataBits, Parity, SerialPort, StopBits};
use std::{time::Duration};

pub struct Status {
    main_volume: u8,
    input: u8,
    standby: bool,
    input_1_effect: u8,
    input_2_effect: u8,
    input_6_effect: u8,
}

pub struct Serial {
    port: Box<dyn SerialPort>,
}

fn connect(port: String) -> Box<dyn SerialPort> {
    serialport::new(port, 57_600)
        .parity(Parity::Odd)
        .data_bits(DataBits::Eight)
        .stop_bits(StopBits::One)
        .timeout(Duration::from_secs(1))
        .open()
        .expect("Failed to open port")
}

impl Serial {
    pub fn new(port: &str) -> Serial {
        Serial {
            port: connect(port.to_string()),
        }
    }

    pub fn read(&mut self, size: usize) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![0; size];
        let received = self.port.read(buf.as_mut_slice()).unwrap();
        assert!(size == received, "{} == {}", size, received);
        return buf
    }

    pub fn write(&mut self, buf: &[u8]) {
        let written = self.port.write(buf).unwrap();
        assert!(buf.len() == written)
    }

    pub fn volume_up(&mut self) {
        let data = [0x08];
        self.write(&data);
    }

    pub fn volume_down(&mut self) {
        let data = [0x09];
        self.write(&data);
    }

    pub fn mute(&mut self) {

    }

    pub fn select_input(&mut self, input: u8) {
        let data = [0x08];
        self.write(&data);
    }

    pub fn turn_on(&mut self) {
        let data = [0x11, 0x11, 0x14, 0x39, 0x38, 0x30, 0x39];
        self.write(&data);
    }

    pub fn turn_off(&mut self) {
        let data = [0x30, 0x37, 0x36];
        self.write(&data);
    }

    pub fn status(&mut self) -> Status {
        let data = [0x34];
        self.write(&data);

        let buf = self.read(24);

        assert!(buf[0] == 0xAA);
        assert!(buf[1] == 0x0A);
        assert!(buf[2] == 0x14);

        println!("{:?}", buf);

        let status = Status {
            main_volume: buf[3],
            input: buf[7],
            standby: buf[20] == 0x01,
            input_1_effect: buf[13],
            input_2_effect: buf[11],
            input_6_effect: buf[12],
        };

        return status
    }
}
