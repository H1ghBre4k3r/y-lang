use crate::{
    codegen::{CodeGen, CodegenContext},
    parser::ast::{Instance, TypeName},
    typechecker::ValidatedTypeInformation,
};

impl<'ctx> CodeGen<'ctx> for Instance<ValidatedTypeInformation> {
    type ReturnValue = ();

    fn codegen(&self, ctx: &CodegenContext<'ctx>) -> Self::ReturnValue {
        let Instance {
            name,
            functions,
            declarations,
            ..
        } = self;

        // Instance methods are methods associated with a specific type
        // For each function in the instance block, we need to:
        // 1. Generate the LLVM function with an implicit 'this' parameter
        // 2. Register the function in a way that it can be called on instances of the type
        
        let type_name = match name {
            TypeName::Literal(name, _) => name,
            _ => {
                panic!("Instance blocks can only be applied to literal type names, not complex types");
            }
        };
        
        // Process function implementations
        for function in functions {
            // Generate the function with 'this' parameter injection
            // The function name will be something like "TypeName_methodName"
            let method_name = format!("{}_{}", type_name, function.id.name);
            
            // For now, we'll generate the function normally
            // TODO: Add 'this' parameter injection and method dispatch
            function.codegen(ctx);
        }
        
        // Process method declarations (external methods)
        for _declaration in declarations {
            // Method declarations are just forward declarations
            // They don't need implementation here - they're handled elsewhere
            // TODO: Implement method declaration registration
        }
    }
}