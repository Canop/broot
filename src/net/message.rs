use {
    crate::{
        errors::NetError,
        command::Sequence,
    },
    std::{
        io::{
            self,
            BufRead,
            Write,
        },
    },
};

/// A message which may be sent by a
#[derive(Debug)]
pub enum Message {
    Command(String),
    Hi,
    //GetRoot
    Sequence(Sequence),
}

fn read_line<BR: BufRead>(r: &mut BR) -> Result<String, NetError> {
    let mut line = String::new();
    r.read_line(&mut line)?;
    debug!("read line => {:?}", &line);
    while line.ends_with('\n') || line.ends_with('\r') {
        line.pop();
    }
    Ok(line)
}

impl Message {
    pub fn read<BR: BufRead>(r: &mut BR) -> Result<Self, NetError> {
        // the first line gives the type of message
        match read_line(r)?.as_ref() {
            "CMD" => Ok(Self::Command(
                read_line(r)?,
            )),
            "SEQ" => Ok(Self::Sequence(Sequence::new(
                read_line(r)?,
                read_line(r)?,
            ))),
            _ => Err(NetError::InvalidMessage),
        }
    }
    pub fn write<W: Write>(&self, w: &mut W) -> io::Result<()> {
        match self {
            Self::Command(c) => {
                writeln!(w, "CMD")?;
                writeln!(w, "{}", c)
            }
            Self::Hi => {
                writeln!(w, "HI")
            }
            Self::Sequence(Sequence { separator, raw }) => {
                writeln!(w, "SEQ")?;
                writeln!(w, "{}", separator)?;
                writeln!(w, "{}", raw)
            }
        }
    }
}
