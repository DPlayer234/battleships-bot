pub trait Encode 
where Self: Sized {
	type Out;
	type In: ?Sized;

	fn encode(&self) -> Self::Out;
	fn try_decode(data: &Self::In) -> Option<Self>;
}
