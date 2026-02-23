(* let board = Array.make_matrix 9 9 0 *)
let problem_board = [|
6; 5; 9; 0; 1; 0; 2; 8; 0;
1; 0; 0; 0; 5; 0; 0; 3; 0;
2; 0; 0; 8; 0; 0; 0; 1; 0;
0; 0; 0; 1; 3; 5; 0; 7; 0;
8; 0; 0; 9; 0; 0; 0; 0; 2;
0; 0; 3; 0; 7; 8; 6; 4; 0;
3; 0; 2; 0; 0; 9; 0; 0; 4;
0; 0; 0; 0; 0; 1; 8; 0; 0;
0; 0; 8; 7; 6; 0; 0; 0; 0;
|] [@@ocamlformat "disable"]

let solved_board = Array.copy problem_board
let get_item board (x, y) = board.(x + (y * 9))
let next_item (x, y) = if x < 8 then (x + 1, y) else (0, y + 1)
let set_item board (x, y) value = board.(x + (y * 9)) <- value

let get_as_string board (x, y) =
  let i = get_item board (x, y) in
  if i > 0 then string_of_int i else "."

(* Grabs a list of valid values *)
let valid_values board (x, y) =
  let valid_predicates_array = Array.make 10 true in
  for i = 0 to 8 do
    valid_predicates_array.(get_item board (x, i)) <- false;
    valid_predicates_array.(get_item board (i, y)) <- false
  done;
  let small_square_x = x - (x mod 3) and small_square_y = y - (y mod 3) in
  for x = small_square_x to small_square_x + 2 do
    for y = small_square_y to small_square_y + 2 do
      valid_predicates_array.(get_item board (x, y)) <- false
    done
  done;
  let valid_list = ref [] in
  for i = 1 to 9 do
    if valid_predicates_array.(i) then valid_list := i :: !valid_list
  done;
  !valid_list (* i always forget ! is deref *)

(* i would like to call this line recursively actually *)

let rec fill board (x, y) =
  if y > 8 then true
  else if get_item board (x, y) > 0 then fill board (next_item (x, y))
  else
    match valid_values board (x, y) with
    | [] -> false
    (* Basically everytime you call this function the head kind of shrinks *)
    | valid_values -> try_entry board (x, y) valid_values

and try_entry board (x, y) = function
  | [] ->
    set_item board (x, y) 0;
    false (*Oh god we are cooked*)
  | curr :: rest ->
      set_item board (x, y) curr;
      if fill board (next_item (x, y)) then true
      else try_entry board (x, y) rest

let print_board board =
  for y = 0 to 8 do
    for x = 0 to 8 do
      print_string (get_as_string board (x, y));
      print_string " "
    done;
    print_newline ()
  done

let _ = fill solved_board (0, 0)

let () =
  print_endline "Problem board:";
  print_board problem_board;
  print_endline "solved board:";
  print_board solved_board
