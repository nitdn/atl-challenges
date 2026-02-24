let _problem_board_medium =
  [
  6; 5; 9; 0; 1; 0; 2; 8; 0;
  1; 0; 0; 0; 5; 0; 0; 3; 0;
  2; 0; 0; 8; 0; 0; 0; 1; 0;
  0; 0; 0; 1; 3; 5; 0; 7; 0;
  8; 0; 0; 9; 0; 0; 0; 0; 2;
  0; 0; 3; 0; 7; 8; 6; 4; 0;
  3; 0; 2; 0; 0; 9; 0; 0; 4;
  0; 0; 0; 0; 0; 1; 8; 0; 0;
  0; 0; 8; 7; 6; 0; 0; 0; 0;
] [@ocamlformat "disable"]

let _problem_board_hard =
  [
  3; 0; 0; 0; 4; 9; 0; 0; 0; (* Row 1 *)
  0; 0; 0; 6; 0; 0; 5; 0; 1; (* Row 2 *)
  7; 5; 2; 0; 0; 1; 0; 0; 0; (* Row 3 *)
  0; 0; 1; 0; 0; 0; 7; 0; 0; (* Row 4 *)
  5; 0; 0; 3; 9; 6; 0; 0; 0; (* Row 5 *)
  0; 0; 8; 1; 5; 0; 0; 9; 6; (* Row 6 *)
  0; 0; 3; 0; 1; 0; 0; 6; 0; (* Row 7 *)
  0; 0; 4; 0; 0; 0; 1; 0; 0; (* Row 8 *)
  0; 0; 0; 0; 2; 8; 0; 0; 0  (* Row 9 *)
] [@ocamlformat "disable"]

let problem_board = _problem_board_hard

let set_item board entry_index value =
  List.mapi
    (fun index element -> if index = entry_index then value else element)
    board

let diff list1 list2 =
  List.filter (fun element -> not (List.mem element list2)) list1

let valid_values board entry_index =
  let numbers = [ 1; 2; 3; 4; 5; 6; 7; 8; 9 ] in
  let filter_rows numbers =
    let row = List.filteri (fun idx _ -> idx / 9 = entry_index / 9) board in
    diff numbers row
  in
  let filter_columns numbers =
    let column =
      List.filteri (fun idx _ -> idx mod 9 = entry_index mod 9) board
    in
    diff numbers column
  in
  let filter_boxes numbers =
    let get_box idx =
      let row = idx / 9 in
      let column = idx mod 9 in
      (row / 3 * 3) + (column / 3)
    in
    let box =
      List.filteri (fun idx _ -> get_box idx = get_box entry_index) board
    in
    diff numbers box
  in
  (* Grabs a list of valid values *)
  numbers |> filter_rows |> filter_columns |> filter_boxes

let rec fill board =
  let rec try_entry board idx = function
    | [] -> None (*Oh god we are cooked*)
    | curr :: rest -> (
        let board = set_item board idx curr in
        match fill board with
        | Some _ as pass -> pass
        | _ -> try_entry board idx rest)
  in
  (* Find empty cell *)
  match List.find_index (fun value -> value = 0) board with
  (* Filled *)
  | None -> Some board
  | Some idx ->
      (* i would like to call this line recursively actually *)
      try_entry board idx (valid_values board idx)

let get_as_string value = if value > 0 then string_of_int value else "."

let print_board board =
  List.iteri
    (fun idx value ->
      print_string (get_as_string value);
      if idx mod 9 = 9 - 1 (* last row *) then
        if idx mod (9 * 3) = (9 * 3) - 1 then
          print_endline "\n------+-------+------"
        else print_newline ()
      else if idx mod 3 = 2 then print_string " | "
      else print_string " ")
    board

let solved_board =
  match fill problem_board with Some board -> board | None -> failwith "bad board"

let () =
  print_endline "Problem board:";
  print_board problem_board;
  print_endline "solved board:";
  print_board solved_board
