use inkwell::AddressSpace;

use crate::{
    codegen::{CodeGen, CodegenContext},
    parser::ast::Constant,
    typechecker::ValidatedTypeInformation,
};

impl<'ctx> CodeGen<'ctx> for Constant<ValidatedTypeInformation> {
    type ReturnValue = ();

    fn codegen(&self, ctx: &CodegenContext<'ctx>) {
        let Constant {
            id,
            type_name: _,
            value,
            ..
        } = self;

        // Generate code for the constant value
        let Some(constant_value) = value.codegen(ctx) else {
            panic!("Constant value must produce a value");
        };

        // For constants, we typically create global variables or compile-time constants
        // Since we're working with LLVM IR generation, we'll create a global variable
        let constant_name = &id.name;

        // Get the LLVM type for the constant from the value's type information
        let value_type_id = &value.get_info().type_id;
        let llvm_type = ctx.get_llvm_type(value_type_id);
        let basic_type = crate::codegen::convert_metadata_to_basic(llvm_type)
            .expect("Constant type must be basic");

        // Create a global variable for the constant
        let global_variable =
            ctx.module
                .add_global(basic_type, Some(AddressSpace::from(0)), constant_name);

        // Set the global variable as constant (immutable)
        global_variable.set_constant(true);
        global_variable.set_linkage(inkwell::module::Linkage::Internal);

        // Initialize the global variable with the constant value
        global_variable.set_initializer(&constant_value);

        // Store the constant in the scope so it can be referenced
        // Note: We need to handle this differently from regular variables since globals are accessed differently
        ctx.store_constant(constant_name, global_variable.as_pointer_value().into());
    }
}
