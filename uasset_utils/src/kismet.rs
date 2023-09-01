macro_rules! inst_member {
    (@inst $name:ident) => {
        $name: Box<Expr>,
    };
    (@vec $name:ident) => {
        $name: Vec<Expr>,
    };
}

macro_rules! inst_walk {
    (@ $ex:ident $member_name:ident : Expr) => {
        walk_macro(&$ex.$member_name);
    };
    (@ $ex:ident $member_name:ident : Box<Expr>) => {
        walk_macro(&$ex.$member_name);
    };
    (@ $ex:ident $member_name:ident : Vec<Expr>) => {
        for $ex in $ex.$member_name { walk_macro(&$ex); }
    };
    (@ $ex:ident $member_name:ident : $tp:ty) => {
    };
    ($ex:ident, $member_name:ident : $($tp:tt)*) => {
        inst_walk!(@ $ex $member_name : $($tp)*)
    };
}
macro_rules! build_walk {
    ($ex:ident, $( $name:ident, { $( $member_name:ident : { $($member_type:tt)* } )* } )* ) => {
        fn walk_macro(ex: &Expr) {
            match ex {
                $( Expr::$name($ex) => {
                    $(inst_walk!($ex, $member_name : $($member_type)*);)*
                }, )*
            }
        }
    };
}

macro_rules! inst {
    ($name:ident, $( $member_name:ident: [ $($member_type:tt)* ] ),* ) => {
        pub struct $name {
            $( $member_name: $($member_type)*, )*
        }
    };
}

macro_rules! for_each2 {
    ( $( $name:ident { $( $member_name:ident : [ $($member_type:tt)* ] )* } )* ) => {
        pub enum Expr {
            $( $name($name), )*
        }
        $( inst!($name, $($member_name : [$($member_type)*]),* );)*
        build_walk!(ex, $( $name, { $( $member_name : { $($member_type)* } ) } )*);
    };
}

/*
for_each_entry!(
    ExLocalVariable {
        //a: String,
        b: Vec<Expr>,
        c: Box<Expr>
    },
    ExInstanceVariable {
        a: Vec<Expr>,
    },
    ExDefaultVariable {
        b: Box<Expr>
    },
);
*/

mod a {
    for_each2!(
        ExLocalVariable {
            //a: [ Expr ]
            b: [ Vec<Expr> ]
            c: [ Box<Expr> ]
        }
        ExInstanceVariable {
            a: [ Vec<Expr> ]
        }
        ExDefaultVariable {
            b: [Box<Expr>]
        }
    );
}

//for_each!( ExLocalVariable, ExInstanceVariable, ExDefaultVariable,);

/*
for_each!(
    ExLocalVariable {},
    ExInstanceVariable {},
    ExDefaultVariable {},
    ExReturn {},
    ExJump {},
    ExJumpIfNot {},
    ExAssert {},
    ExNothing {},
    ExLet {},
    ExClassContext {},
    ExMetaCast {},
    ExLetBool {},
    ExEndParmValue {},
    ExEndFunctionParms {},
    ExSelf {},
    ExSkip {},
    ExContext {},
    ExContextFailSilent {},
    ExVirtualFunction {},
    ExFinalFunction {},
    ExIntConst {},
    ExFloatConst {},
    ExStringConst {},
    ExObjectConst {},
    ExNameConst {},
    ExRotationConst {},
    ExVectorConst {},
    ExByteConst {},
    ExIntZero {},
    ExIntOne {},
    ExTrue {},
    ExFalse {},
    ExTextConst {},
    ExNoObject {},
    ExTransformConst {},
    ExIntConstByte {},
    ExNoInterface {},
    ExDynamicCast {},
    ExStructConst {},
    ExEndStructConst {},
    ExSetArray {},
    ExEndArray {},
    ExPropertyConst {},
    ExUnicodeStringConst {},
    ExInt64Const {},
    ExUInt64Const {},
    ExPrimitiveCast {},
    ExSetSet {},
    ExEndSet {},
    ExSetMap {},
    ExEndMap {},
    ExSetConst {},
    ExEndSetConst {},
    ExMapConst {},
    ExEndMapConst {},
    ExStructMemberContext {},
    ExLetMulticastDelegate {},
    ExLetDelegate {},
    ExLocalVirtualFunction {},
    ExLocalFinalFunction {},
    ExLocalOutVariable {},
    ExDeprecatedOp4A {},
    ExInstanceDelegate {},
    ExPushExecutionFlow {},
    ExPopExecutionFlow {},
    ExComputedJump {},
    ExPopExecutionFlowIfNot {},
    ExBreakpoint {},
    ExInterfaceContext {},
    ExObjToInterfaceCast {},
    ExEndOfScript {},
    ExCrossInterfaceCast {},
    ExInterfaceToObjCast {},
    ExWireTracepoint {},
    ExSkipOffsetConst {},
    ExAddMulticastDelegate {},
    ExClearMulticastDelegate {},
    ExTracepoint {},
    ExLetObj {},
    ExLetWeakObjPtr {},
    ExBindDelegate {},
    ExRemoveMulticastDelegate {},
    ExCallMulticastDelegate {},
    ExLetValueOnPersistentFrame {},
    ExArrayConst {},
    ExEndArrayConst {},
    ExSoftObjectConst {},
    ExCallMath {},
    ExSwitchValue {},
    ExInstrumentationEvent {},
    ExArrayGetByRef {},
    ExClassSparseDataVariable {},
    ExFieldPathConst {},
);
*/
/*
for_each!(
    ExLocalVariable,
    ExInstanceVariable,
    ExDefaultVariable,
    ExReturn,
    ExJump,
    ExJumpIfNot,
    ExAssert,
    ExNothing,
    ExLet,
    ExClassContext,
    ExMetaCast,
    ExLetBool,
    ExEndParmValue,
    ExEndFunctionParms,
    ExSelf,
    ExSkip,
    ExContext,
    ExContextFailSilent,
    ExVirtualFunction,
    ExFinalFunction,
    ExIntConst,
    ExFloatConst,
    ExStringConst,
    ExObjectConst,
    ExNameConst,
    ExRotationConst,
    ExVectorConst,
    ExByteConst,
    ExIntZero,
    ExIntOne,
    ExTrue,
    ExFalse,
    ExTextConst,
    ExNoObject,
    ExTransformConst,
    ExIntConstByte,
    ExNoInterface,
    ExDynamicCast,
    ExStructConst,
    ExEndStructConst,
    ExSetArray,
    ExEndArray,
    ExPropertyConst,
    ExUnicodeStringConst,
    ExInt64Const,
    ExUInt64Const,
    ExPrimitiveCast,
    ExSetSet,
    ExEndSet,
    ExSetMap,
    ExEndMap,
    ExSetConst,
    ExEndSetConst,
    ExMapConst,
    ExEndMapConst,
    ExStructMemberContext,
    ExLetMulticastDelegate,
    ExLetDelegate,
    ExLocalVirtualFunction,
    ExLocalFinalFunction,
    ExLocalOutVariable,
    ExDeprecatedOp4A,
    ExInstanceDelegate,
    ExPushExecutionFlow,
    ExPopExecutionFlow,
    ExComputedJump,
    ExPopExecutionFlowIfNot,
    ExBreakpoint,
    ExInterfaceContext,
    ExObjToInterfaceCast,
    ExEndOfScript,
    ExCrossInterfaceCast,
    ExInterfaceToObjCast,
    ExWireTracepoint,
    ExSkipOffsetConst,
    ExAddMulticastDelegate,
    ExClearMulticastDelegate,
    ExTracepoint,
    ExLetObj,
    ExLetWeakObjPtr,
    ExBindDelegate,
    ExRemoveMulticastDelegate,
    ExCallMulticastDelegate,
    ExLetValueOnPersistentFrame,
    ExArrayConst,
    ExEndArrayConst,
    ExSoftObjectConst,
    ExCallMath,
    ExSwitchValue,
    ExInstrumentationEvent,
    ExArrayGetByRef,
    ExClassSparseDataVariable,
    ExFieldPathConst,
);
*/
