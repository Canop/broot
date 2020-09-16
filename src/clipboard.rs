use crate::errors::ClipboardError;
use copypasta::ClipboardProvider;

pub fn set_string( content: String ) -> Result< (), ClipboardError >
{
	let mut clipboard_context = copypasta::ClipboardContext::new()?;
	Ok( clipboard_context.set_contents( content )? )
}

pub fn get_string() -> Result< String, ClipboardError >
{
	let mut clipboard_context = copypasta::ClipboardContext::new()?;
	Ok( clipboard_context.get_contents()? )
}

