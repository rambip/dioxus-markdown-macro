use dioxus::prelude::*;

const TETRIS_WIDTH : usize = 6;
const TETRIS_HEIGHT : usize = 4;
const N_COUPS: usize = TETRIS_HEIGHT * TETRIS_WIDTH / 4;

const N_COULEURS: usize = 7;

#[derive(PartialEq, Clone, Copy)]
enum Couleur {
    T=0,
    S=1,
    Z=2,
    O=3,
    I=4,
    L=5,
    J=6,
}

impl Couleur {
    fn paint(&self) -> &'static str {
        match self {
            Couleur::T => "purple",
            Couleur::S => "green",
            Couleur::Z => "red",
            Couleur::O => "yellow",
            Couleur::I => "cyan",
            Couleur::L => "orange",
            Couleur::J => "blue",
        }
    }
}

struct EtatJeu {
    grille: [[bool; TETRIS_WIDTH]; TETRIS_HEIGHT],
    pieces_jouees: Vec<u8>,
    // TODO: retirer
    couleur_jouees: [u8; N_COULEURS],
}




const N_PIECES: usize = 19;
// Chaque piece existe en plusieurs versions selon l'orientation
// - position relative des 3 blocs adjacents
// - couleur
static PIECES: [([(i32, i32); 3], Couleur); N_PIECES] = [
    ([( 0,  1), (-1,  1), (-1,  2)], Couleur::S), // S
    ([( 1,  0), ( 1,  1), ( 2,  1)], Couleur::S), // S'
    ([( 0,  1), ( 1,  1), ( 1,  2)], Couleur::Z), // Z
    ([( 0,  1), ( 1,  0), (-1,  1)], Couleur::Z), // Z'
    ([( 0,  1), ( 0,  2), ( 1,  1)], Couleur::T), // T1
    ([( 1,  0), ( 1,  1), ( 2,  0)], Couleur::T), // T2
    ([( 0,  1), (-1,  1), ( 0,  2)], Couleur::T), // T3
    ([(-1,  1), ( 0,  1), ( 1,  1)], Couleur::T), // T4
    ([( 0,  1), (-1,  1), (-2,  1)], Couleur::J), // J1
    ([( 0,  1), ( 0,  2), ( 1,  2)], Couleur::J), // J2
    ([( 1,  0), ( 0,  1), ( 2,  0)], Couleur::J), // J3
    ([( 1,  0), ( 1,  1), ( 1,  2)], Couleur::J), // J4
    ([( 1,  0), ( 2,  0), ( 2,  1)], Couleur::L), // L1
    ([( 0,  1), ( 0,  2), (-1,  2)], Couleur::L), // L2
    ([( 0,  1), ( 1,  1), ( 2,  1)], Couleur::L), // L3
    ([( 0,  1), ( 0,  2), ( 1,  0)], Couleur::L), // L4
    ([( 1,  0), ( 2,  0), ( 3,  0)], Couleur::I), // I
    ([( 0,  1), ( 0,  2), ( 0,  3)], Couleur::I), // I'
    ([( 0,  1), ( 1,  0), ( 1,  1)], Couleur::O),
];


#[derive(Copy, Clone, Debug)]
struct Coup {
    // position sur la grille
    i: i32,
    j: i32,
    // la piece posee est un index dans PIECES
    piece: usize,
}


impl Coup {
    // donne les 4 blocs touches sur le plateau par le coup indique
    // si le coup n'est pas valide, les blocs peuvent sortir de la grille !
    fn blocs_touches(&self) -> [(i32, i32); 4] {
        let (blocs, _couleur) = &PIECES[self.piece];

        let mut result = [(0, 0); 4];
        result[0] = (self.i, self.j);

        for i in 0..=2 {
            result[i+1] = (
                (self.i + blocs[i].0),
                (self.j + blocs[i].1) 
            );
        };
        
        result
    }
}


fn case_vide(g: [[bool; TETRIS_WIDTH]; TETRIS_HEIGHT]) -> (usize, usize) {
    for j in 0..TETRIS_WIDTH {
        for i in 0..TETRIS_HEIGHT {
            if g[i][j] == false {
                return (i, j);
            }
        }
    }
    panic!("pas de case vide !!!");
}

/// Prend une grille de blocs, un coup et indique si le coup est valide pour ce plateau de jeu.
fn valide(etat: &EtatJeu, coup: Coup) -> bool {
    // pour chaque bloc de la piece, on regarde a quel endroit elle atterit sur la grille.
    // si la piece sort de la grille ou qu'il y a une collision, le coup est invalide
    for (i, j) in coup.blocs_touches() {
        if i < 0 || i>=(TETRIS_HEIGHT as i32) || j < 0 || j>=(TETRIS_WIDTH as i32) {return false};

        if etat.grille[i as usize][j as usize] {return false};

    };

    true
}


fn jouer_coup(etat: &mut EtatJeu, coup: Coup) {
    let couleur = PIECES[coup.piece].1;
    for (i, j) in coup.blocs_touches() {
        etat.grille[i as usize][j as usize] = true;
    }
    etat.pieces_jouees.push(coup.piece as u8);
    etat.couleur_jouees[couleur as usize]+=1;
}

fn annuler_coup(etat: &mut EtatJeu, coup: Coup) {
    let couleur = PIECES[coup.piece].1;
    for (i,j) in coup.blocs_touches() {
        etat.grille[i as usize][j as usize] = false;
    }
    etat.pieces_jouees.pop();
    etat.couleur_jouees[couleur as usize]-=1;

}


// donne la liste de tous les coups pouvant etre joues a ce tour:
fn coup_valides(etat: &EtatJeu) -> Vec<Coup> {
    // on joue sur la prochaine case libre:
    let mut r = Vec::with_capacity(N_PIECES);
    let (i, j) = case_vide(etat.grille);
    for piece in 0..N_PIECES {
        let coup = Coup {i:i as i32, j:j as i32, piece};
        if valide(etat, coup){
            r.push(coup);
        }
    }
    r
}


fn liste_perfect_clear(etat: &mut EtatJeu, acc: &mut Vec<Vec<u8>>) {
    if etat.pieces_jouees.len() == N_COUPS {
        if true { //etat.couleur_jouees.iter().all(|&x| x>=1 && x <= 2) {
            // si toutes les couleurs de pieces on ete posees
            acc.push(etat.pieces_jouees.clone());
        }
        return;
    }

    for coup in coup_valides(etat) {
        jouer_coup(etat, coup);

        liste_perfect_clear(etat, acc);

        annuler_coup(etat, coup);
    }
}

struct Model {
    possibilities : Option<Vec<Vec<u8>>>,
    n: usize,
    width: u32,
    height: u32,
    size_bloc: i64,
}


enum Msg {
    NextPossibility,
    PrevPossibility,
    Compute,
    JumpToPossibility(f32),
}


fn display_tetris(state: &[u8], size_bloc: i64) -> Element {
    // genere la grille a partir des indexes des pieces posees
    let mut grille = [[false; TETRIS_WIDTH]; TETRIS_HEIGHT];

    // indexe des pieces 
    let mut grille_pieces = [[None; TETRIS_WIDTH]; TETRIS_HEIGHT];

    // liste de composants svg (rectangles)
    let mut shapes = Vec::new();

    for (num, &i_piece) in state.iter().enumerate() {

        // reconstruit le coup suivant
        let (i, j) = case_vide(grille);
        let coup : Coup = Coup { i: i as i32, j: j as i32, piece: i_piece as usize};

        // affiche les differents blocs de ce tetramino
        for (i, j) in coup.blocs_touches().into_iter() {
            grille[i as usize][j as usize] = true;
            grille_pieces[i as usize][j as usize] = Some(num);

            let x = size_bloc*(j as i64);
            let y = size_bloc*(i as i64);

            let color = PIECES[i_piece as usize].1.paint();

            shapes.push(
                rsx!{
                    rect{
                        x:x,
                        y:y,
                        width:size_bloc,
                        height:size_bloc,
                        fill:color,
                    }
                }
            );

            // indique si la case (i', j') adjacente a besoin d'une bordure:
            // seulement si elle est issue d'un tetramino different
            let contour = |i_ad: i32, j_ad: i32| {
                // Vérification si i_ad et j_ad sont positifs
                if i_ad >= 0 && j_ad >= 0 {
                    // Vérification si l'indexation donne un Some(n)
                    match grille_pieces.get(i_ad  as usize).and_then(|row| row.get((j_ad as usize))) {
                        Some(Some(n)) if *n != num => true,
                        _ => false
                    }
                } else {
                    false // Si i_ad ou j_ad sont négatifs, retourner false
                }
            };

            // traduit deux coordonées en ligne SVG
            let line = |x1: i64, y1: i64, x2: i64, y2: i64| rsx!{
                line {
                    x1: x1,
                    y1: y1,
                    x2: x2,
                    y2: y2,
                    stroke_width: 2,
                    stroke: "white",
                }
            };


            if contour(i-1, j) {
                shapes.push(line(x, y, x+size_bloc, y));
            }
            if contour(i+1, j) {
                shapes.push(line(x, y+size_bloc, x+size_bloc, y+size_bloc));
            }
            if contour(i, j-1) {
                shapes.push(line(x, y, x, y+size_bloc));
            }
            if contour(i, j+1) {
                shapes.push(line(x+size_bloc, y, x+size_bloc, y+size_bloc));
            }
        }
    }

    rsx!{
        {shapes.into_iter()}
    }
}


#[component]
pub fn PerfectClear(
    size_bloc: i64,
) -> Element {
    let mut index = use_signal(|| 0);
    let mut possibilities = Vec::new();
    let mut etat_vide: EtatJeu = EtatJeu {
        grille: [[false; TETRIS_WIDTH]; TETRIS_HEIGHT],
        pieces_jouees: Vec::new(),
        couleur_jouees: [0; N_COULEURS],
    };

    liste_perfect_clear(&mut etat_vide, &mut possibilities);
    rsx! {
        div {
            svg {
                {display_tetris(&possibilities[index()], size_bloc)}
            }
            div {
                "Combinaison {index} / {possibilities.len()}"
            }
            input {
                r#type: "range",
                min: 0,
                max: 1,
                step: 0.000001,
                oninput: move |e| { 
                    let r = e.value().parse::<f32>().unwrap() * (possibilities.len() -1) as f32;
                    index.set(r as usize);
                }
            }
        }
    }
}