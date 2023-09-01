macro_rules! build_walk {
    ($ex:ident, $member_name:ident : Box<Expr>) => {
        walk_expression(&$ex.$member_name);
    };
    ($ex:ident, $member_name:ident : Vec<Expr>) => {
        for $ex in $ex.$member_name.iter() { walk_expression(&$ex); }
    };
    ($ex:ident, $member_name:ident : $tp:ty) => {
    };
}

macro_rules! expression {
    ($name:ident, $( $member_name:ident: [ $($member_type:tt)* ] ),* ) => {
        pub struct $name {
            $( $member_name: $($member_type)*, )*
        }
    };
}

macro_rules! for_each {
    ( $( $name:ident { $( $member_name:ident : [ $($member_type:tt)* ] )* } )* ) => {
        pub enum Expr {
            $( $name($name), )*
        }
        $( expression!($name, $($member_name : [$($member_type)*]),* );)*
        fn walk_expression(ex: &Expr) {
            match ex {
                $( Expr::$name(ex) => {
                    $(build_walk!(ex, $member_name : $($member_type)*);)*
                }, )*
            }
        }
    };
}

struct KismetPropertyPointer;
struct PackageIndex;
struct FName;
struct OrderedFloat<T>(T);
struct Vector<T>(T);
struct Transform<T>(T);
struct FScriptText;
struct ECastToken;
struct KismetSwitchCase;
struct EScriptInstrumentationType;

for_each!(
    ExLocalVariable { variable: [ KismetPropertyPointer ] }
    ExInstanceVariable { variable: [ KismetPropertyPointer ] }
    ExDefaultVariable { variable: [ KismetPropertyPointer ] }
    ExReturn { return_expression: [ Box<Expr> ] }
    ExJump { code_offset: [ u32 ] }
    ExJumpIfNot { code_offset: [ u32 ] boolean_expression: [ Box<Expr> ] }
    ExAssert { line_number: [ u16 ] debug_mode: [ bool ] assert_expression: [ Box<Expr> ] }
    ExNothing {  }
    ExLet { value: [ KismetPropertyPointer ] variable: [ Box<Expr> ] expression: [ Box<Expr> ] }
    ExClassContext { object_expression: [ Box<Expr> ] offset: [ u32 ] r_value_pointer: [ KismetPropertyPointer ] context_expression: [ Box<Expr> ] }
    ExMetaCast { class_ptr: [ PackageIndex ] target_expression: [ Box<Expr> ] }
    ExLetBool { variable_expression: [ Box<Expr> ] assignment_expression: [ Box<Expr> ] }
    ExEndParmValue {  }
    ExEndFunctionParms {  }
    ExSelf {  }
    ExSkip { code_offset: [ u32 ] skip_expression: [ Box<Expr> ] }
    ExContext { object_expression: [ Box<Expr> ] offset: [ u32 ] r_value_pointer: [ KismetPropertyPointer ] context_expression: [ Box<Expr> ] }
    ExContextFailSilent { object_expression: [ Box<Expr> ] offset: [ u32 ] r_value_pointer: [ KismetPropertyPointer ] context_expression: [ Box<Expr> ] }
    ExVirtualFunction { virtual_function_name: [ FName ] parameters: [ Vec<Expr> ] }
    ExFinalFunction { stack_node: [ PackageIndex ] parameters: [ Vec<Expr> ] }
    ExIntConst {  }
    ExFloatConst { value: [ OrderedFloat<f32> ] }
    ExStringConst { value: [ String ] }
    ExObjectConst { value: [ PackageIndex ] }
    ExNameConst { value: [ FName ] }
    ExRotationConst { rotator: [ Vector<OrderedFloat<f64>> ] }
    ExVectorConst { value: [ Vector<OrderedFloat<f64>> ] }
    ExByteConst {  }
    ExIntZero {  }
    ExIntOne {  }
    ExTrue {  }
    ExFalse {  }
    ExTextConst { value: [ Box<FScriptText> ] }
    ExNoObject {  }
    ExTransformConst { value: [ Transform<OrderedFloat<f64>> ] }
    ExIntConstByte {  }
    ExNoInterface {  }
    ExDynamicCast { class_ptr: [ PackageIndex ] target_expression: [ Box<Expr> ] }
    ExStructConst { struct_value: [ PackageIndex ] struct_size: [ i32 ] value: [ Vec<Expr> ] }
    ExEndStructConst {  }
    ExSetArray { assigning_property: [ Option<Box<Expr>> ] array_inner_prop: [ Option<PackageIndex> ] elements: [ Vec<Expr> ] }
    ExEndArray {  }
    ExPropertyConst { property: [ KismetPropertyPointer ] }
    ExUnicodeStringConst { value: [ String ] }
    ExInt64Const {  }
    ExUInt64Const {  }
    ExPrimitiveCast { conversion_type: [ ECastToken ] target: [ Box<Expr> ] }
    ExSetSet { set_property: [ Box<Expr> ] elements: [ Vec<Expr> ] }
    ExEndSet {  }
    ExSetMap { map_property: [ Box<Expr> ] elements: [ Vec<Expr> ] }
    ExEndMap {  }
    ExSetConst { inner_property: [ KismetPropertyPointer ] elements: [ Vec<Expr> ] }
    ExEndSetConst {  }
    ExMapConst { key_property: [ KismetPropertyPointer ] value_property: [ KismetPropertyPointer ] elements: [ Vec<Expr> ] }
    ExEndMapConst {  }
    ExStructMemberContext { struct_member_expression: [ KismetPropertyPointer ] struct_expression: [ Box<Expr> ] }
    ExLetMulticastDelegate { variable_expression: [ Box<Expr> ] assignment_expression: [ Box<Expr> ] }
    ExLetDelegate { variable_expression: [ Box<Expr> ] assignment_expression: [ Box<Expr> ] }
    ExLocalVirtualFunction { virtual_function_name: [ FName ] parameters: [ Vec<Expr> ] }
    ExLocalFinalFunction { stack_node: [ PackageIndex ] parameters: [ Vec<Expr> ] }
    ExLocalOutVariable { variable: [ KismetPropertyPointer ] }
    ExDeprecatedOp4A {  }
    ExInstanceDelegate { function_name: [ FName ] }
    ExPushExecutionFlow { pushing_address: [ u32 ] }
    ExPopExecutionFlow {  }
    ExComputedJump { code_offset_expression: [ Box<Expr> ] }
    ExPopExecutionFlowIfNot { boolean_expression: [ Box<Expr> ] }
    ExBreakpoint {  }
    ExInterfaceContext { interface_value: [ Box<Expr> ] }
    ExObjToInterfaceCast { class_ptr: [ PackageIndex ] target: [ Box<Expr> ] }
    ExEndOfScript {  }
    ExCrossInterfaceCast { class_ptr: [ PackageIndex ] target: [ Box<Expr> ] }
    ExInterfaceToObjCast { class_ptr: [ PackageIndex ] target: [ Box<Expr> ] }
    ExWireTracepoint {  }
    ExSkipOffsetConst {  }
    ExAddMulticastDelegate { delegate: [ Box<Expr> ] delegate_to_add: [ Box<Expr> ] }
    ExClearMulticastDelegate { delegate_to_clear: [ Box<Expr> ] }
    ExTracepoint {  }
    ExLetObj { variable_expression: [ Box<Expr> ] assignment_expression: [ Box<Expr> ] }
    ExLetWeakObjPtr { variable_expression: [ Box<Expr> ] assignment_expression: [ Box<Expr> ] }
    ExBindDelegate { function_name: [ FName ] delegate: [ Box<Expr> ] object_term: [ Box<Expr> ] }
    ExRemoveMulticastDelegate { delegate: [ Box<Expr> ] delegate_to_add: [ Box<Expr> ] }
    ExCallMulticastDelegate { stack_node: [ PackageIndex ] parameters: [ Vec<Expr> ] delegate: [ Box<Expr> ] }
    ExLetValueOnPersistentFrame { destination_property: [ KismetPropertyPointer ] assignment_expression: [ Box<Expr> ] }
    ExArrayConst { inner_property: [ KismetPropertyPointer ] elements: [ Vec<Expr> ] }
    ExEndArrayConst {  }
    ExSoftObjectConst { value: [ Box<Expr> ] }
    ExCallMath { stack_node: [ PackageIndex ] parameters: [ Vec<Expr> ] }
    ExSwitchValue { end_goto_offset: [ u32 ] index_term: [ Box<Expr> ] default_term: [ Box<Expr> ] cases: [ Vec<KismetSwitchCase> ] }
    ExInstrumentationEvent { event_type: [ EScriptInstrumentationType ] event_name: [ Option<FName> ] }
    ExArrayGetByRef { array_variable: [ Box<Expr> ] array_index: [ Box<Expr> ] }
    ExClassSparseDataVariable { variable: [ KismetPropertyPointer ] }
    ExFieldPathConst { value: [ Box<Expr> ] }
);
