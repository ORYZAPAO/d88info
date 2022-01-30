use std::io::Read;

use crate::report_d88::ReportD88;

impl ReportD88 {
    pub fn report_d88_noinfo(&mut self, mut fh: std::fs::File) {
        let mut buffer = Vec::<u8>::new();

        if let Ok(size) = fh.read_to_end(&mut buffer) {
            let mut ofst: usize = 0;
            let mut buf16: [u8; 16] = [0; 16];

            self.print_title_bar();

            while ofst < size {
                for idx in 0..16 {
                    buf16[idx] = buffer[ofst + idx];
                }

                self.print_16byte(&buf16, ofst, ansi_term::Color::White);
                println!("");

                ofst += 16;
            }
        }
    }
}
