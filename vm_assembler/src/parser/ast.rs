use std::str::FromStr;

#[derive(Debug, Default)]
pub(crate) struct Ast {
    nodes: Vec<AstNode>,
}

impl Ast {
    pub fn push(&mut self, node: AstNode) {
        self.nodes.push(node);
    }
}

#[derive(Debug)]
pub(crate) enum AstNode {
    ResolvedToken(ResolvedToken),
    UnresolvedToken(UnResolvedToken),
}

#[derive(Debug)]
pub(crate) enum ResolvedToken {
    U16,
    Register,
    KeyWord,
}

#[derive(Debug)]
pub(crate) enum UnResolvedToken {
    Label,
}

impl FromStr for AstNode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if let Ok(_num) = s.parse::<u16>() {
            return Ok(AstNode::ResolvedToken(ResolvedToken::U16));
        }
        if let Ok(_keyword) = super::tokens::KeyWord::from_str(s) {
            return Ok(AstNode::ResolvedToken(ResolvedToken::KeyWord));
        }
        if let Some(idx) = s.find(',') {
            let _reg = vm_cpu::registers::Register::from_str(&s[0..idx]).map_err(|_| ())?;
            return Ok(AstNode::ResolvedToken(ResolvedToken::Register));
        }
        if let Ok(_reg) = vm_cpu::registers::Register::from_str(s) {
            Ok(AstNode::ResolvedToken(ResolvedToken::Register))
        } else {
            let _label = super::tokens::Label::from_str(&s[0..s.len()]).map_err(|_| ())?;
            Ok(AstNode::UnresolvedToken(UnResolvedToken::Label))
        }
    }
}
