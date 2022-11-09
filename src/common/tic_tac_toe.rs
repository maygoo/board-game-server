use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Preamble(ClientState),
    WaitTurn,
    YourTurn,
    Move((Piece, usize, usize)),
    InvalidMove(String),
    // Nought or Cross piece means they win
    // Empty piece means game is over i.e. disconnect
    GameOver(End),
    // temporary, move to standard server message
    Status(super::ServerStatus),
}

impl From<Vec<u8>> for Message {
    fn from(bytes: Vec<u8>) -> Self {
        bincode::deserialize(&bytes).unwrap()
    }
}

impl From<Message> for Vec<u8> {
    fn from(message: Message) -> Self {
        bincode::serialize(&message).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum End {
    Victory(Piece),
    Draw,
    Disconnect,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum Piece {
    Nought,
    Cross,
    Empty
}

impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Nought => write!(f, "O"),
            Self::Cross  => write!(f, "X"),
            Self::Empty  => write!(f, " "),
        }
    }
}

pub struct ServerState {
    pub board: Board,
    pub turn: Turn,
    pub current_player: Piece,
    pub crosses_player: usize,
    pub noughts_player: usize,
    pub winner: End,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientState {
    pub board: Board,
    pub turn: Turn,
    pub opponent: String,
    pub piece: Piece,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
pub enum Turn {
    Begin,
    TurnStart,
    TurnWait,
    End,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Board {
    grid: Vec<Vec<Piece>>,
    pub size: usize,
}

pub const NAME: &str = "Tic Tac Toe";
pub const BOARD_SIZE: usize = 3;
pub const INSTRUCTIONS: &str = "
  Wait until your turn then
  enter two numbers, eg 1 2
  for your move. 1 1 starts
  at the the top left cell.
";

impl Board {
    pub fn new(size: usize) -> Self {
        Board {
            grid: vec![vec![Piece::Empty; size]; size],
            size,
        }
    }
    
    // forefully place the piece on the board
    // useful for the client because the move
    // is already validated by the server
    pub fn place(&mut self, p: Piece, x: usize, y: usize) {
        self.grid[y][x] = p;
    }

    pub fn try_place(&mut self, p: Piece, x: usize, y:usize) -> Result<(Piece, usize, usize), String> {
        // check if cell is empty then do move
        match &mut self.grid[y][x] {
            Piece::Empty => {
                self.place(p.clone(), x, y);
                Ok((p, x, y))
            }
            p => Err(format!("{} {} already has a {p} on it! Enter another move", (y + 65) as u8 as char, x+1)), // quick convert idxs to game coords
        }
    }
    
    pub fn check_victory(&self, piece: Piece) -> Option<End> {
        // check for horizontal victory
        let mut win = self.grid.iter().filter(|row| {
            row.iter().filter(|cell| {
                **cell == piece
            }).count() == self.size
        }).count();

        // TODO reduce by using zip function ? maybe
        // check for vertical victory
        for i in 0..self.size {
            let mut flag = true;
            for j in 0..self.size {
                if self.grid[j][i] != piece { flag = false; break; }
            }
            if flag {
                win = 1;
                break;
            }
        }

        // check for diag victory
        let mut flag1 = true;
        let mut flag2 = true;
        for i in 0..self.size {
            if self.grid[i][i] != piece { flag1 = false; }
            if self.grid[i][self.size-i-1] != piece { flag2 = false; }

            if !flag1 && !flag2 { break; }
        }
        if flag1 || flag2 { win = 1; }

        // if no one has won, check to see if there is a draw
        if win < 1 {
            if self.check_draw() {
                Some(End::Draw)
            } else {
                None
            }
        } else {
            Some(End::Victory(piece))
        }
    }

    fn check_draw(&self) -> bool {
        !self.grid.iter()
            .map(|row| row.contains(&Piece::Empty))
            .any(|p| p)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Vec<Piece>> {
        self.grid.iter()
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut out = String::new();

        // create the horizontal separator
        // based on the board size
        let sep = "-".repeat(self.size * 2 - 1);
    
        // ascii offset to convert numbers to letters
        let offset = 65;
    
        out += "  ";
        for i in 1..=self.size {
            out += &format!("{i} ");
        }
        out += "\n";
    
        for (i, row) in self.grid.iter().enumerate() {
            if i > 0 { out += format!("  {}\n", sep).as_str() }
    
            for (j, cell) in row.iter().enumerate() {
                if j == 0 { out += format!("{} ", char::from((i+offset) as u8)).as_str() }
                if j > 0 { out += "|" }
    
                out += format!("{cell}").as_str();
            }
    
            out += "\n";
        }
        write!(f, "{}", out)
    }
}

impl ClientState {
    pub fn new(opponent: String, piece: Piece, board_size: usize) -> Self {
        ClientState {
            // we don't need to send the entire grid to the client
            // the client can initialise its own empty grid
            board: Board {
                grid: Vec::new(),
                size: board_size,
            },
            turn: Turn::Begin,
            opponent,
            piece,
        }
    }
}

impl ServerState {
    pub fn new(board_size: usize) -> Self {
        ServerState {
            board: Board::new(board_size),
            turn: Turn::Begin,
            winner: End::Disconnect,
            current_player: Piece::Cross,
            noughts_player: 0,
            crosses_player: 1,
        }
    }
}

impl Piece {
    pub fn next(self) -> Self {
        match self {
            Piece::Cross => Piece::Nought,
            Piece::Nought => Piece::Cross,
            Piece::Empty => unreachable!(),
        }
    }
}
