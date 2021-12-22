use crate::*;



pub enum Type {
    Basic(Ident),
    //AnonymousEnum(EnumData),
    AnonymousStruct(StructData),
    AnonymousUnion(UnionData),
}
