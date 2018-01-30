
use std::env;
use std::rc::Rc;
use std::path::Path;

use error::*;
use traits::*;
use expressions::*;
use ast::*;
use objects::*;
use input::*;


#[derive(Debug)]
pub enum TemplateSource<'a> {
    TemplatePathSource(&'a Path),
    TemplateSourceString(&'a str),
    TemplateSource(Rc<Template>),
    DocumentSource(Document)
}

impl<'a, P: AsRef<Path>> From<&'a P> for TemplateSource<'a> {
    fn from(source: &'a P) -> Self {
        TemplateSource::TemplatePathSource(source.as_ref())
    }
}

// impl<'a> From<&'a AsRef<Template>> for TemplateSource<'a> {
//     fn from(source: &'a AsRef<Template>) -> Self {
//         TemplateSource::TemplateSource(source.as_ref())
//     }    
// }

impl<'a> From<&'a str> for TemplateSource<'a> {
    fn from(source: &'a str) -> Self {
        TemplateSource::TemplateSourceString(source)
    }
}

impl<'a> From<Rc<Template>> for TemplateSource<'a> {
    fn from(source: Rc<Template>) -> Self {
        TemplateSource::TemplateSource(source)
    }
}

impl From<Document> for TemplateSource<'static> {
    fn from(doc: Document) -> Self {
        TemplateSource::DocumentSource(doc)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocumentProvider(Rc<Document>);

impl Default for DocumentProvider {
    fn default() -> Self {
        let source_path = env::var_os("DEFAULT_PAGE")
            .unwrap_or_else(|| "./res/tests/app/todomvc/app.ism".into());
        let source_path = Path::new(&source_path);

        let res = DocumentProvider::create(&source_path);

        if let Err(ref e) = res {
            eprintln!("Error when processing document: {:?}\n", e);
            // let bt = Backtrace::new();
            // eprintln!("Backtrace:\n{:?}", bt);

            panic!("Cannot process document");
        };

        res.unwrap()
    }
}

impl DocumentProvider {
    pub fn create<'a, S: Into<TemplateSource<'a>>>(source: S) -> DocumentProcessingResult<DocumentProvider> {
        let template = from_source(source)?;

        Ok(DocumentProvider(Rc::new(template)))
    }

    pub fn doc<'a>(&'a self) -> &'a Document { self.0.as_ref() }
}

fn from_source<'a, S: Into<TemplateSource<'a>>>(source: S) -> DocumentProcessingResult<Document> {
    let source = source.into();

    match source {
        TemplateSource::TemplatePathSource(ref source_path) => {
            let template = parser::parse_file(source_path)?;
            let mut ctx: DefaultProcessingContext<ProcessedExpression> = DefaultProcessingContext::for_template(Rc::new(template.clone()));
            TryProcessFrom::try_process_from(&template, &mut ctx)
        }

        TemplateSource::TemplateSourceString(ref src) => {
            let template = parser::parse_str(src)?;
            let mut ctx: DefaultProcessingContext<ProcessedExpression> = DefaultProcessingContext::for_template(Rc::new(template.clone()));
            TryProcessFrom::try_process_from(&template, &mut ctx)            
        }

        TemplateSource::TemplateSource(template) => {
            let mut ctx: DefaultProcessingContext<ProcessedExpression> = DefaultProcessingContext::for_template(template.clone());
            TryProcessFrom::try_process_from(template.as_ref(), &mut ctx)
        }

        TemplateSource::DocumentSource(document) => Ok(document)
    }
}