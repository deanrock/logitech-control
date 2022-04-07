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

use serde::{Deserialize, Serialize};
use serialport::{DataBits, Parity, SerialPort, SerialPortInfo, StopBits, UsbPortInfo};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Status {
    main_volume: u8,
    input: u8,
    standby: bool,
    input_1_effect: u8,
    input_2_effect: u8,
    input_6_effect: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Effect {
    Effect3d = 0x14,
    Effect4_1 = 0x15,
    Effect2_1 = 0x16,
    Disabled = 0x35,
}
pub struct Serial {
    port: Box<dyn SerialPort>,
    cached_status: Option<Status>,
}

pub fn find_port() -> Option<String> {
    for item in serialport::available_ports().unwrap() {
        let SerialPortInfo {
            port_name,
            port_type,
        } = item;
        if let serialport::SerialPortType::UsbPort(UsbPortInfo {
            vid: _,
            pid: _,
            serial_number: _,
            manufacturer,
            product: _,
        }) = port_type
        {
            if manufacturer == Some("FTDI".to_string()) {
                return Some(port_name);
            }
        }
    }

    return None;
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
    pub fn new(port: String) -> Serial {
        Serial {
            port: connect(port),
            cached_status: None,
        }
    }

    pub fn read(&mut self, size: usize) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![0; size];
        let mut start = 0;
        while size > start {
            let slice = buf.get_mut(start..size).unwrap();
            let received = self.port.read(slice).unwrap();
            start += received;
        }

        return buf;
    }

    pub fn write(&mut self, buf: &[u8]) {
        let written = self.port.write(buf).unwrap();
        assert!(buf.len() == written)
    }

    pub fn volume_up(&mut self) {
        let data = [0x08];
        self.write(&data);

        self.read(1);
    }

    pub fn volume_down(&mut self) {
        let data = [0x09];
        self.write(&data);

        self.read(1);
    }

    pub fn mute(&mut self) {}

    pub fn select_input(&mut self, input: u8) {
        let data = [0x08];
        self.write(&data);
    }

    pub fn select_effect(&mut self, effect: Effect) {
        let u = effect as u8;
        let data = [u];
        self.write(&data);
        assert!(self.read(1) == data);
    }

    pub fn configuration_reset(&mut self) {
        let data = [
            0xAA, 0x0E, // = type
            0x03, // = length of remaining data (excluding checksum)
            0x20, 0x00, 0x00, 0xCF, // = checksum
            0xAA, 0x0A, // = type
            0x14, // = length of remaining data (excluding checksum)
            0x0A, 0x15, 0x15, 0x15, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00,
            0x06, 0x01, 0x03, 0x00, 0x00, 0x00, 0x8C, // = checksum
            0x36,
        ];
        self.write(&data);

        assert!(self.read(6) == [0xAA, 0xFF, 0x01, 0x8A, 0x76, 0x36]);
    }

    pub fn turn_on(&mut self) {
        let data = [0x11, 0x11, 0x14, 0x39, 0x38, 0x30, 0x39];
        self.write(&data);

        self.read(7);
    }

    pub fn turn_off(&mut self) {
        let data = [0x30, 0x37, 0x36];
        self.write(&data);

        self.read(3);
    }

    pub fn cached_status(&mut self) -> Status {
        if let Some(status) = self.cached_status {
            return status;
        }

        return self.status();
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
            input: buf[7] + 1, // map 0->5 to 1->6
            standby: buf[20] == 0x01,
            input_1_effect: buf[13],
            input_2_effect: buf[11],
            input_6_effect: buf[12],
        };

        self.cached_status = Some(status);

        return status;
    }
}
