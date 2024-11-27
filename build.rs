use std::fs;
use std::path::Path;
use std::io::Write;

fn main() {
    let src_pages_dir = Path::new("src/pages");
    let output_dir = src_pages_dir.to_path_buf();

    // Parcourt tous les fichiers Markdown dans src/pages
    for entry in fs::read_dir(src_pages_dir).expect("Impossible de lire le répertoire src/pages") {
        let entry = entry.expect("Erreur lors de la lecture d'une entrée");
        let path = entry.path();

        // Vérifie que c'est un fichier Markdown
        if path.extension().map_or(false, |ext| ext == "md") {
            process_markdown_file(&path, &output_dir);
        }
    }

    println!("Traitement des fichiers Markdown terminé");
}

fn process_markdown_file(md_path: &Path, output_dir: &Path) {
    // Lit le contenu du fichier Markdown
    let md_content = fs::read_to_string(md_path)
        .expect("Impossible de lire le fichier Markdown");

    // Transforme le chemin de .md à .rs
    let rs_path = output_dir.join(
        md_path.file_name()
            .expect("Nom de fichier invalide")
            .to_str()
            .expect("Conversion du nom de fichier impossible")
            .replace(".md", ".rs")
    );

    // Exemple de transformation du contenu (à personnaliser selon vos besoins)
    let processed_content = transform_markdown_content(&md_content);

    // Écrit le contenu transformé dans le fichier .rs
    let mut rs_file = fs::File::create(&rs_path)
        .expect("Impossible de créer le fichier de sortie");
    
    rs_file.write_all(processed_content.as_bytes())
        .expect("Erreur lors de l'écriture du fichier");

    println!("Traité {} -> {}", 
        md_path.display(), 
        rs_path.display()
    );
}

fn transform_markdown_content(content: &str) -> String {
    // Exemple de transformation simple
    // Vous pouvez personnaliser cette fonction selon vos besoins spécifiques
    let rsx = parse_markdown::parse(content);
    let content = dioxus_autofmt::write_block_out(&rsx)
        .expect("can't indent generated rsx");
    format!("rsx! {{ \n{content}\n }}")
}

// Ajoute cette ligne à Cargo.toml dans la section [package]
// build = "build.rs"
