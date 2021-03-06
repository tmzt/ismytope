use std::rc::Rc;

use isymtope_ast_common::*;
use isymtope_build::*;
use isymtope_generate::*;

fn parse_template(src: &str) -> IsymtopeGenerateResult<DocumentProvider> {
    let template = Rc::new(parser::parse_str(&src)?);

    // Create document provider
    let mut ctx: DefaultProcessingContext<ProcessedExpression> =
        DefaultProcessingContext::for_template(template.clone());

    let document: Document = TryProcessFrom::try_process_from(template.as_ref(), &mut ctx)?;
    let document_provider = DocumentProvider::create(document)?;

    Ok(document_provider)
}

pub fn compile_template(src: &str, base_url: &str) -> IsymtopeGenerateResult<String> {
    let template = parse_template(&src)?;
    let mut template_context = DefaultTemplateContext::new(Rc::new(template));

    let app_name = "/app.ism";
    let template_path = "/app.ism";
    let path = "/";

    let req = TemplateRequestMsg::RenderAppRoute(
        base_url.to_string(),
        app_name.to_string(),
        template_path.to_string(),
        path.to_string(),
    );

    let response = template_context.handle_msg(req)?;
    let TemplateResponseMsg::RenderComplete(result) = response;
    let body = result.into_inner();

    Ok(body)
}
