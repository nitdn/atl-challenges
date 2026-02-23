let board = Array.make_matrix 9 9 0
let print_row row = Format.printf "@[%a@.@]" Format.(pp_print_array ~pp_sep:pp_print_space pp_print_int) row
let print_board board = Array.iter print_row board
let () = print_board board;

