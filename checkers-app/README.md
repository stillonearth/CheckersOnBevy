## 🗀 checkers-app

### 🎮 Front-End

`bevy_frontend.rs` implements Bevy application which is influenced by [1].

Bevy uses ECS pattern to describe game logic. It suggests to organize logic in following manner:

- _Entities_ — game engine objects such as meshes, groups of meshes
- _Components_ — attributes that can be matched to entities
- _Systems_ — functions that operate on entities and their components

In Bevy there is also notion on _Resources_ which are similar to global variables or singletons. _Events_ are used to pass messages between systems. _Plugins_ organize Systems and Resources.

**Plugins and Systems**

_BoardPlugin_ describes game board and high-level events such as selection of square, movement and game termination.

```
 BoardPlugin
  |
  |--[Resources ]
  |    |--SelectedSquare
  |    `--SelectedPiece
  |
  |--[Startup Systems]
  |    `--create_board
  |
  |--[Events]
  |   `--event_square_selected
  |
  |--[Components]
  |    |--Square
  |    `--Piece
  |
  |--[Systems]
  |    |--player_move
  |    |     `--update_entity_pieces
  |    |
  |    |
  |    |--computer_move
  |    `--check_game_termination
  |
  `--[Plugins]
       `--PiecesPlugin
```

_PiecesPlugin_ describes pieces, movement animation and highlighting.

```
 PiecesPlugin
  |
  |--[Startup Systems]
  |    `--create_pieces
  |
  |--[Events]
  |   `--event_piece_moved
  |
  |--[Systems]
  |    `--highlight_piece
  |
  `--[Plugins]
       `--TweeningPlugin
```

_UIPlugin_ describes buttons and game state text label.

```
 UIPlugin
  |
  |--[Startup Systems]
  |    `--init_text
  |
  |--[Events]
  |   `--event_piece_moved
  |
  `--[Systems]
       |--next_move_text_update
       `--button_system
```