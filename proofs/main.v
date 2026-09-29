Require Import List.
Require Import String.
Require Import BinNat.
Require Import ZArith.
From Wasm Require Import bytes numerics datatypes host.
From WasmVerifier Require Import Assertions Verifier.

Definition Vi32 i := VAL_int32 (Wasm_int.int_of_Z i32m i).
Definition Vi64 i := VAL_int64 (Wasm_int.int_of_Z i64m i).
Definition Mt l et := {|modtab_type := {|tt_limits := l; tt_elem_type := et|}|}.
Definition Mm l := {|modmem_type := l|}.
Definition Mg mut t init := {|modglob_type := {|tg_mut := mut; tg_t := t|}; modglob_init := init|}.

Definition Mi m n d := {|
  imp_module := list_byte_of_string m;
  imp_name := list_byte_of_string n;
  imp_desc := d;
|}.

Definition Me n d := {|
  modexp_name := list_byte_of_string n;
  modexp_desc := d;
|}.

Definition Ma ofs al := {|memarg_offset := ofs; memarg_align := al|}.

Definition minimum_liquidity : module_func := {|
  modfunc_type := 0%N;
  modfunc_locals := nil;
  modfunc_body :=
    BI_const_num (Vi32 1000) ::
    BI_return ::
    BI_unreachable ::
    nil;
|}.

Definition get_amount_out : module_func := {|
  modfunc_type := 1%N;
  modfunc_locals := T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil;
  modfunc_body :=
    BI_global_get 0%N ::
    BI_const_num (Vi32 64) ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_local_tee 10%N (*__frame_ptr*) ::
    BI_global_set 0%N ::
    BI_local_get 10%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 0%N 3%N) ::
    BI_local_get 10%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 8%N 3%N) ::
    BI_local_get 10%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 16%N 3%N) ::
    BI_local_get 10%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 24%N 3%N) ::
    BI_local_get 10%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 32%N 3%N) ::
    BI_local_get 10%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 40%N 3%N) ::
    BI_local_get 10%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 48%N 3%N) ::
    BI_local_get 10%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 56%N 3%N) ::
    BI_const_num (Vi32 10000) ::
    BI_local_set 4%N (*D*) ::
    BI_local_get 0%N (*amount_in*) ::
    BI_const_num (Vi32 0) ::
    BI_relop T_i32 (Relop_i (ROI_gt SX_U)) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 1%N (*reserve_in*) ::
    BI_const_num (Vi32 0) ::
    BI_relop T_i32 (Relop_i (ROI_gt SX_U)) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 2%N (*reserve_out*) ::
    BI_const_num (Vi32 0) ::
    BI_relop T_i32 (Relop_i (ROI_gt SX_U)) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 3%N (*fee_bps*) ::
    BI_local_get 4%N (*D*) ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 4%N (*D*) ::
    BI_local_get 3%N (*fee_bps*) ::
    BI_local_set 12%N ::
    BI_local_tee 11%N ::
    BI_local_get 12%N ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 11%N ::
    BI_local_get 12%N ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_local_set 5%N (*g*) ::
    BI_local_get 10%N (*__frame_ptr*) ::
    BI_local_get 0%N (*amount_in*) ::
    BI_local_get 5%N (*g*) ::
    BI_call 18%N ::
    BI_local_get 10%N (*__frame_ptr*) ::
    BI_local_set 6%N (*ag*) ::
    BI_local_get 10%N (*__frame_ptr*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 6%N (*ag*) ::
    BI_local_get 2%N (*reserve_out*) ::
    BI_call 21%N ::
    BI_local_get 10%N (*__frame_ptr*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 7%N (*num*) ::
    BI_local_get 10%N (*__frame_ptr*) ::
    BI_const_num (Vi32 32) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 1%N (*reserve_in*) ::
    BI_local_get 4%N (*D*) ::
    BI_call 18%N ::
    BI_local_get 10%N (*__frame_ptr*) ::
    BI_const_num (Vi32 32) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 8%N (*ri_d*) ::
    BI_local_get 10%N (*__frame_ptr*) ::
    BI_const_num (Vi32 48) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 8%N (*ri_d*) ::
    BI_local_get 6%N (*ag*) ::
    BI_call 19%N ::
    BI_local_get 10%N (*__frame_ptr*) ::
    BI_const_num (Vi32 48) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 9%N (*den*) ::
    BI_local_get 7%N (*num*) ::
    BI_local_get 9%N (*den*) ::
    BI_call 28%N ::
    BI_local_get 10%N (*__frame_ptr*) ::
    BI_const_num (Vi32 64) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_global_set 0%N ::
    BI_return ::
    BI_unreachable ::
    nil;
|}.

Definition get_amount_in : module_func := {|
  modfunc_type := 2%N;
  modfunc_locals := T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil;
  modfunc_body :=
    BI_global_get 0%N ::
    BI_const_num (Vi32 48) ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_local_tee 10%N (*__frame_ptr*) ::
    BI_global_set 0%N ::
    BI_local_get 10%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 0%N 3%N) ::
    BI_local_get 10%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 8%N 3%N) ::
    BI_local_get 10%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 16%N 3%N) ::
    BI_local_get 10%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 24%N 3%N) ::
    BI_local_get 10%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 32%N 3%N) ::
    BI_local_get 10%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 40%N 3%N) ::
    BI_const_num (Vi32 10000) ::
    BI_local_set 4%N (*D*) ::
    BI_local_get 0%N (*amount_out*) ::
    BI_const_num (Vi32 0) ::
    BI_relop T_i32 (Relop_i (ROI_gt SX_U)) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 1%N (*reserve_in*) ::
    BI_const_num (Vi32 0) ::
    BI_relop T_i32 (Relop_i (ROI_gt SX_U)) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 2%N (*reserve_out*) ::
    BI_const_num (Vi32 0) ::
    BI_relop T_i32 (Relop_i (ROI_gt SX_U)) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 0%N (*amount_out*) ::
    BI_local_get 2%N (*reserve_out*) ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 3%N (*fee_bps*) ::
    BI_local_get 4%N (*D*) ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 4%N (*D*) ::
    BI_local_get 3%N (*fee_bps*) ::
    BI_local_set 12%N ::
    BI_local_tee 11%N ::
    BI_local_get 12%N ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 11%N ::
    BI_local_get 12%N ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_local_set 5%N (*g*) ::
    BI_local_get 10%N (*__frame_ptr*) ::
    BI_local_get 1%N (*reserve_in*) ::
    BI_local_get 0%N (*amount_out*) ::
    BI_call 18%N ::
    BI_local_get 10%N (*__frame_ptr*) ::
    BI_local_set 6%N (*rx*) ::
    BI_local_get 10%N (*__frame_ptr*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 6%N (*rx*) ::
    BI_local_get 4%N (*D*) ::
    BI_call 21%N ::
    BI_local_get 10%N (*__frame_ptr*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 7%N (*num*) ::
    BI_local_get 10%N (*__frame_ptr*) ::
    BI_const_num (Vi32 32) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 2%N (*reserve_out*) ::
    BI_local_get 0%N (*amount_out*) ::
    BI_local_set 12%N ::
    BI_local_tee 11%N ::
    BI_local_get 12%N ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 11%N ::
    BI_local_get 12%N ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_local_get 5%N (*g*) ::
    BI_call 18%N ::
    BI_local_get 10%N (*__frame_ptr*) ::
    BI_const_num (Vi32 32) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 8%N (*den*) ::
    BI_local_get 7%N (*num*) ::
    BI_local_get 8%N (*den*) ::
    BI_call 28%N ::
    BI_local_set 9%N (*q*) ::
    BI_local_get 9%N (*q*) ::
    BI_const_num (Vi32 1) ::
    BI_local_tee 12%N ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_tee 13%N ::
    BI_local_get 12%N ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 13%N ::
    BI_local_get 10%N (*__frame_ptr*) ::
    BI_const_num (Vi32 48) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_global_set 0%N ::
    BI_return ::
    BI_unreachable ::
    nil;
|}.

Definition quote : module_func := {|
  modfunc_type := 3%N;
  modfunc_locals := T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil;
  modfunc_body :=
    BI_global_get 0%N ::
    BI_const_num (Vi32 32) ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_local_tee 5%N (*__frame_ptr*) ::
    BI_global_set 0%N ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 0%N 3%N) ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 8%N 3%N) ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 16%N 3%N) ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 24%N 3%N) ::
    BI_local_get 0%N (*amount_a*) ::
    BI_const_num (Vi32 0) ::
    BI_relop T_i32 (Relop_i (ROI_gt SX_U)) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 1%N (*reserve_a*) ::
    BI_const_num (Vi32 0) ::
    BI_relop T_i32 (Relop_i (ROI_gt SX_U)) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 2%N (*reserve_b*) ::
    BI_const_num (Vi32 0) ::
    BI_relop T_i32 (Relop_i (ROI_gt SX_U)) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_local_get 0%N (*amount_a*) ::
    BI_local_get 2%N (*reserve_b*) ::
    BI_call 18%N ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_local_set 3%N (*num*) ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 1%N (*reserve_a*) ::
    BI_call 17%N ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 4%N (*den*) ::
    BI_local_get 3%N (*num*) ::
    BI_local_get 4%N (*den*) ::
    BI_call 28%N ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_const_num (Vi32 32) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_global_set 0%N ::
    BI_return ::
    BI_unreachable ::
    nil;
|}.

Definition mint_initial : module_func := {|
  modfunc_type := 4%N;
  modfunc_locals := T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil;
  modfunc_body :=
    BI_global_get 0%N ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_local_tee 5%N (*__frame_ptr*) ::
    BI_global_set 0%N ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 0%N 3%N) ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 8%N 3%N) ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_local_get 0%N (*amount0*) ::
    BI_local_get 1%N (*amount1*) ::
    BI_call 18%N ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_local_set 2%N (*p*) ::
    BI_local_get 2%N (*p*) ::
    BI_call 29%N ::
    BI_local_set 3%N (*s*) ::
    BI_local_get 3%N (*s*) ::
    BI_call 0%N ::
    BI_local_set 7%N ::
    BI_local_tee 6%N ::
    BI_local_get 7%N ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 6%N ::
    BI_local_get 7%N ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_local_set 4%N (*minted*) ::
    BI_local_get 4%N (*minted*) ::
    BI_const_num (Vi32 0) ::
    BI_relop T_i32 (Relop_i (ROI_gt SX_U)) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 4%N (*minted*) ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_global_set 0%N ::
    BI_return ::
    BI_unreachable ::
    nil;
|}.

Definition mint_proportional : module_func := {|
  modfunc_type := 5%N;
  modfunc_locals := T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil;
  modfunc_body :=
    BI_global_get 0%N ::
    BI_const_num (Vi32 64) ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_local_tee 14%N (*__frame_ptr*) ::
    BI_global_set 0%N ::
    BI_local_get 14%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 0%N 3%N) ::
    BI_local_get 14%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 8%N 3%N) ::
    BI_local_get 14%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 16%N 3%N) ::
    BI_local_get 14%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 24%N 3%N) ::
    BI_local_get 14%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 32%N 3%N) ::
    BI_local_get 14%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 40%N 3%N) ::
    BI_local_get 14%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 48%N 3%N) ::
    BI_local_get 14%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 56%N 3%N) ::
    BI_local_get 2%N (*reserve0*) ::
    BI_const_num (Vi32 0) ::
    BI_relop T_i32 (Relop_i (ROI_gt SX_U)) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 3%N (*reserve1*) ::
    BI_const_num (Vi32 0) ::
    BI_relop T_i32 (Relop_i (ROI_gt SX_U)) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 4%N (*total_supply*) ::
    BI_const_num (Vi32 0) ::
    BI_relop T_i32 (Relop_i (ROI_gt SX_U)) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 14%N (*__frame_ptr*) ::
    BI_local_get 0%N (*amount0*) ::
    BI_local_get 4%N (*total_supply*) ::
    BI_call 18%N ::
    BI_local_get 14%N (*__frame_ptr*) ::
    BI_local_set 5%N (*n0*) ::
    BI_local_get 14%N (*__frame_ptr*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 2%N (*reserve0*) ::
    BI_call 17%N ::
    BI_local_get 14%N (*__frame_ptr*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 6%N (*d0*) ::
    BI_local_get 14%N (*__frame_ptr*) ::
    BI_const_num (Vi32 32) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 1%N (*amount1*) ::
    BI_local_get 4%N (*total_supply*) ::
    BI_call 18%N ::
    BI_local_get 14%N (*__frame_ptr*) ::
    BI_const_num (Vi32 32) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 7%N (*n1*) ::
    BI_local_get 14%N (*__frame_ptr*) ::
    BI_const_num (Vi32 48) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 3%N (*reserve1*) ::
    BI_call 17%N ::
    BI_local_get 14%N (*__frame_ptr*) ::
    BI_const_num (Vi32 48) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 8%N (*d1*) ::
    BI_local_get 5%N (*n0*) ::
    BI_local_get 6%N (*d0*) ::
    BI_call 27%N ::
    BI_local_set 9%N (*fits0*) ::
    BI_local_get 7%N (*n1*) ::
    BI_local_get 8%N (*d1*) ::
    BI_call 27%N ::
    BI_local_set 10%N (*fits1*) ::
    BI_local_get 9%N (*fits0*) ::
    BI_if (BT_valtype (Some (T_num T_i32))) (
      BI_const_num (Vi32 1) ::
      nil) (
      BI_local_get 10%N ::
      nil) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_const_num (Vi32 0) ::
    BI_local_set 11%N (*minted*) ::
    BI_local_get 9%N (*fits0*) ::
    BI_if (BT_valtype (Some (T_num T_i32))) (
      BI_local_get 10%N ::
      nil) (
      BI_const_num (Vi32 0) ::
      nil) ::
    BI_if (BT_valtype None) (
      BI_local_get 5%N ::
      BI_local_get 6%N ::
      BI_call 28%N ::
      BI_local_set 12%N ::
      BI_local_get 7%N ::
      BI_local_get 8%N ::
      BI_call 28%N ::
      BI_local_set 13%N ::
      BI_local_get 12%N ::
      BI_local_set 11%N ::
      BI_local_get 13%N ::
      BI_local_get 12%N ::
      BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
      BI_if (BT_valtype None) (
        BI_local_get 13%N ::
        BI_local_set 11%N ::
        nil) (
        nil) ::
      nil) (
      BI_local_get 9%N ::
      BI_if (BT_valtype None) (
        BI_local_get 5%N ::
        BI_local_get 6%N ::
        BI_call 28%N ::
        BI_local_set 11%N ::
        nil) (
        BI_local_get 7%N ::
        BI_local_get 8%N ::
        BI_call 28%N ::
        BI_local_set 11%N ::
        nil) ::
      nil) ::
    BI_local_get 11%N (*minted*) ::
    BI_const_num (Vi32 0) ::
    BI_relop T_i32 (Relop_i (ROI_gt SX_U)) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 11%N (*minted*) ::
    BI_local_get 14%N (*__frame_ptr*) ::
    BI_const_num (Vi32 64) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_global_set 0%N ::
    BI_return ::
    BI_unreachable ::
    nil;
|}.

Definition burn_share : module_func := {|
  modfunc_type := 6%N;
  modfunc_locals := T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil;
  modfunc_body :=
    BI_global_get 0%N ::
    BI_const_num (Vi32 32) ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_local_tee 6%N (*__frame_ptr*) ::
    BI_global_set 0%N ::
    BI_local_get 6%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 0%N 3%N) ::
    BI_local_get 6%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 8%N 3%N) ::
    BI_local_get 6%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 16%N 3%N) ::
    BI_local_get 6%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 24%N 3%N) ::
    BI_local_get 2%N (*total_supply*) ::
    BI_const_num (Vi32 0) ::
    BI_relop T_i32 (Relop_i (ROI_gt SX_U)) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 0%N (*liquidity*) ::
    BI_local_get 2%N (*total_supply*) ::
    BI_relop T_i32 (Relop_i (ROI_le SX_U)) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 6%N (*__frame_ptr*) ::
    BI_local_get 0%N (*liquidity*) ::
    BI_local_get 1%N (*balance*) ::
    BI_call 18%N ::
    BI_local_get 6%N (*__frame_ptr*) ::
    BI_local_set 3%N (*num*) ::
    BI_local_get 6%N (*__frame_ptr*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 2%N (*total_supply*) ::
    BI_call 17%N ::
    BI_local_get 6%N (*__frame_ptr*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 4%N (*den*) ::
    BI_local_get 3%N (*num*) ::
    BI_local_get 4%N (*den*) ::
    BI_call 28%N ::
    BI_local_set 5%N (*out*) ::
    BI_local_get 5%N (*out*) ::
    BI_const_num (Vi32 0) ::
    BI_relop T_i32 (Relop_i (ROI_gt SX_U)) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 5%N (*out*) ::
    BI_local_get 6%N (*__frame_ptr*) ::
    BI_const_num (Vi32 32) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_global_set 0%N ::
    BI_return ::
    BI_unreachable ::
    nil;
|}.

Definition k_holds : module_func := {|
  modfunc_type := 7%N;
  modfunc_locals := T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil;
  modfunc_body :=
    BI_global_get 0%N ::
    BI_const_num (Vi32 80) ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_local_tee 9%N (*__frame_ptr*) ::
    BI_global_set 0%N ::
    BI_local_get 9%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 0%N 3%N) ::
    BI_local_get 9%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 8%N 3%N) ::
    BI_local_get 9%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 16%N 3%N) ::
    BI_local_get 9%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 24%N 3%N) ::
    BI_local_get 9%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 32%N 3%N) ::
    BI_local_get 9%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 40%N 3%N) ::
    BI_local_get 9%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 48%N 3%N) ::
    BI_local_get 9%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 56%N 3%N) ::
    BI_local_get 9%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 64%N 3%N) ::
    BI_local_get 9%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 72%N 3%N) ::
    BI_local_get 2%N (*reserve_in*) ::
    BI_const_num (Vi32 0) ::
    BI_relop T_i32 (Relop_i (ROI_gt SX_U)) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 1%N (*amount_out*) ::
    BI_local_get 3%N (*reserve_out*) ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 9%N (*__frame_ptr*) ::
    BI_local_get 2%N (*reserve_in*) ::
    BI_call 17%N ::
    BI_local_get 9%N (*__frame_ptr*) ::
    BI_local_set 4%N (*ri*) ::
    BI_local_get 9%N (*__frame_ptr*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 0%N (*amount_in*) ::
    BI_call 17%N ::
    BI_local_get 9%N (*__frame_ptr*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 5%N (*ai*) ::
    BI_local_get 9%N (*__frame_ptr*) ::
    BI_const_num (Vi32 32) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 4%N (*ri*) ::
    BI_local_get 5%N (*ai*) ::
    BI_call 19%N ::
    BI_local_get 9%N (*__frame_ptr*) ::
    BI_const_num (Vi32 32) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 6%N (*bal_in*) ::
    BI_local_get 9%N (*__frame_ptr*) ::
    BI_const_num (Vi32 48) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 6%N (*bal_in*) ::
    BI_local_get 3%N (*reserve_out*) ::
    BI_local_get 1%N (*amount_out*) ::
    BI_local_set 11%N ::
    BI_local_tee 10%N ::
    BI_local_get 11%N ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 10%N ::
    BI_local_get 11%N ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_call 21%N ::
    BI_local_get 9%N (*__frame_ptr*) ::
    BI_const_num (Vi32 48) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 7%N (*lhs*) ::
    BI_local_get 9%N (*__frame_ptr*) ::
    BI_const_num (Vi32 64) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 2%N (*reserve_in*) ::
    BI_local_get 3%N (*reserve_out*) ::
    BI_call 18%N ::
    BI_local_get 9%N (*__frame_ptr*) ::
    BI_const_num (Vi32 64) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 8%N (*rhs*) ::
    BI_local_get 8%N (*rhs*) ::
    BI_local_get 7%N (*lhs*) ::
    BI_call 24%N ::
    BI_local_get 9%N (*__frame_ptr*) ::
    BI_const_num (Vi32 80) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_global_set 0%N ::
    BI_return ::
    BI_unreachable ::
    nil;
|}.

Definition k_holds_with_fee : module_func := {|
  modfunc_type := 8%N;
  modfunc_locals := T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil;
  modfunc_body :=
    BI_global_get 0%N ::
    BI_const_num (Vi32 144) ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_local_tee 15%N (*__frame_ptr*) ::
    BI_global_set 0%N ::
    BI_const_num (Vi32 0) ::
    BI_local_set 19%N ::
    BI_loop (BT_valtype None) (
      BI_local_get 15%N ::
      BI_local_get 19%N ::
      BI_binop T_i32 (Binop_i BOI_add) ::
      BI_const_num (Vi64 0) ::
      BI_store T_i64 None (Ma 0%N 3%N) ::
      BI_local_get 15%N ::
      BI_local_get 19%N ::
      BI_binop T_i32 (Binop_i BOI_add) ::
      BI_const_num (Vi64 0) ::
      BI_store T_i64 None (Ma 8%N 3%N) ::
      BI_local_get 19%N ::
      BI_const_num (Vi32 16) ::
      BI_binop T_i32 (Binop_i BOI_add) ::
      BI_local_tee 19%N ::
      BI_const_num (Vi32 144) ::
      BI_relop T_i32 (Relop_i ROI_ne) ::
      BI_br_if 0%N ::
      nil) ::
    BI_const_num (Vi32 10000) ::
    BI_local_set 5%N (*D*) ::
    BI_local_get 4%N (*fee_bps*) ::
    BI_local_get 5%N (*D*) ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 2%N (*reserve_in*) ::
    BI_const_num (Vi32 0) ::
    BI_relop T_i32 (Relop_i (ROI_gt SX_U)) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 1%N (*amount_out*) ::
    BI_local_get 3%N (*reserve_out*) ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 15%N (*__frame_ptr*) ::
    BI_local_get 2%N (*reserve_in*) ::
    BI_call 17%N ::
    BI_local_get 15%N (*__frame_ptr*) ::
    BI_local_set 6%N (*ri*) ::
    BI_local_get 15%N (*__frame_ptr*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 0%N (*amount_in*) ::
    BI_call 17%N ::
    BI_local_get 15%N (*__frame_ptr*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 7%N (*ai*) ::
    BI_local_get 15%N (*__frame_ptr*) ::
    BI_const_num (Vi32 32) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 6%N (*ri*) ::
    BI_local_get 7%N (*ai*) ::
    BI_call 19%N ::
    BI_local_get 15%N (*__frame_ptr*) ::
    BI_const_num (Vi32 32) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 8%N (*bal_in*) ::
    BI_local_get 15%N (*__frame_ptr*) ::
    BI_const_num (Vi32 48) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 8%N (*bal_in*) ::
    BI_local_get 5%N (*D*) ::
    BI_call 21%N ::
    BI_local_get 15%N (*__frame_ptr*) ::
    BI_const_num (Vi32 48) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 9%N (*scaled*) ::
    BI_local_get 15%N (*__frame_ptr*) ::
    BI_const_num (Vi32 64) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 0%N (*amount_in*) ::
    BI_local_get 4%N (*fee_bps*) ::
    BI_call 18%N ::
    BI_local_get 15%N (*__frame_ptr*) ::
    BI_const_num (Vi32 64) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 10%N (*fee*) ::
    BI_local_get 15%N (*__frame_ptr*) ::
    BI_const_num (Vi32 80) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 9%N (*scaled*) ::
    BI_local_get 10%N (*fee*) ::
    BI_call 20%N ::
    BI_local_get 15%N (*__frame_ptr*) ::
    BI_const_num (Vi32 80) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 11%N (*adj*) ::
    BI_local_get 15%N (*__frame_ptr*) ::
    BI_const_num (Vi32 96) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 11%N (*adj*) ::
    BI_local_get 3%N (*reserve_out*) ::
    BI_local_get 1%N (*amount_out*) ::
    BI_local_set 17%N ::
    BI_local_tee 16%N ::
    BI_local_get 17%N ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 16%N ::
    BI_local_get 17%N ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_call 21%N ::
    BI_local_get 15%N (*__frame_ptr*) ::
    BI_const_num (Vi32 96) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 12%N (*lhs*) ::
    BI_local_get 15%N (*__frame_ptr*) ::
    BI_const_num (Vi32 112) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 2%N (*reserve_in*) ::
    BI_local_get 3%N (*reserve_out*) ::
    BI_call 18%N ::
    BI_local_get 15%N (*__frame_ptr*) ::
    BI_const_num (Vi32 112) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 13%N (*rr*) ::
    BI_local_get 15%N (*__frame_ptr*) ::
    BI_const_num (Vi32 128) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 13%N (*rr*) ::
    BI_local_get 5%N (*D*) ::
    BI_call 21%N ::
    BI_local_get 15%N (*__frame_ptr*) ::
    BI_const_num (Vi32 128) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 14%N (*rhs*) ::
    BI_local_get 14%N (*rhs*) ::
    BI_local_get 12%N (*lhs*) ::
    BI_call 24%N ::
    BI_local_get 15%N (*__frame_ptr*) ::
    BI_const_num (Vi32 144) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_global_set 0%N ::
    BI_return ::
    BI_unreachable ::
    nil;
|}.

Definition mul_div : module_func := {|
  modfunc_type := 9%N;
  modfunc_locals := T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil;
  modfunc_body :=
    BI_global_get 0%N ::
    BI_const_num (Vi32 32) ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_local_tee 5%N (*__frame_ptr*) ::
    BI_global_set 0%N ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 0%N 3%N) ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 8%N 3%N) ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 16%N 3%N) ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 24%N 3%N) ::
    BI_local_get 2%N (*denominator*) ::
    BI_const_num (Vi32 0) ::
    BI_relop T_i32 (Relop_i (ROI_gt SX_U)) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_local_get 0%N (*a*) ::
    BI_local_get 1%N (*b*) ::
    BI_call 18%N ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_local_set 3%N (*num*) ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 2%N (*denominator*) ::
    BI_call 17%N ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 4%N (*den*) ::
    BI_local_get 3%N (*num*) ::
    BI_local_get 4%N (*den*) ::
    BI_call 28%N ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_const_num (Vi32 32) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_global_set 0%N ::
    BI_return ::
    BI_unreachable ::
    nil;
|}.

Definition mul_div_up : module_func := {|
  modfunc_type := 10%N;
  modfunc_locals := T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil;
  modfunc_body :=
    BI_global_get 0%N ::
    BI_const_num (Vi32 48) ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_local_tee 7%N (*__frame_ptr*) ::
    BI_global_set 0%N ::
    BI_local_get 7%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 0%N 3%N) ::
    BI_local_get 7%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 8%N 3%N) ::
    BI_local_get 7%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 16%N 3%N) ::
    BI_local_get 7%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 24%N 3%N) ::
    BI_local_get 7%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 32%N 3%N) ::
    BI_local_get 7%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 40%N 3%N) ::
    BI_local_get 2%N (*denominator*) ::
    BI_const_num (Vi32 0) ::
    BI_relop T_i32 (Relop_i (ROI_gt SX_U)) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 7%N (*__frame_ptr*) ::
    BI_local_get 0%N (*a*) ::
    BI_local_get 1%N (*b*) ::
    BI_call 18%N ::
    BI_local_get 7%N (*__frame_ptr*) ::
    BI_local_set 3%N (*num*) ::
    BI_local_get 7%N (*__frame_ptr*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 2%N (*denominator*) ::
    BI_call 17%N ::
    BI_local_get 7%N (*__frame_ptr*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 4%N (*den*) ::
    BI_local_get 3%N (*num*) ::
    BI_local_get 4%N (*den*) ::
    BI_call 28%N ::
    BI_local_set 5%N (*q*) ::
    BI_local_get 7%N (*__frame_ptr*) ::
    BI_const_num (Vi32 32) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 5%N (*q*) ::
    BI_local_get 2%N (*denominator*) ::
    BI_call 18%N ::
    BI_local_get 7%N (*__frame_ptr*) ::
    BI_const_num (Vi32 32) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 6%N (*back*) ::
    BI_local_get 6%N (*back*) ::
    BI_local_get 3%N (*num*) ::
    BI_call 22%N ::
    BI_if (BT_valtype None) (
      BI_local_get 5%N ::
      BI_local_get 7%N ::
      BI_const_num (Vi32 48) ::
      BI_binop T_i32 (Binop_i BOI_add) ::
      BI_global_set 0%N ::
      BI_return ::
      nil) (
      nil) ::
    BI_local_get 5%N (*q*) ::
    BI_const_num (Vi32 1) ::
    BI_local_tee 9%N ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_tee 10%N ::
    BI_local_get 9%N ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 10%N ::
    BI_local_get 7%N (*__frame_ptr*) ::
    BI_const_num (Vi32 48) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_global_set 0%N ::
    BI_return ::
    BI_unreachable ::
    nil;
|}.

Definition model_widen : module_func := {|
  modfunc_type := 11%N;
  modfunc_locals := T_num T_i64 :: T_num T_i64 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil;
  modfunc_body :=
    BI_const_num (Vi64 0) ::
    BI_local_set 1%N (*r*) ::
    BI_const_num (Vi64 1) ::
    BI_local_set 2%N (*bit*) ::
    BI_const_num (Vi32 0) ::
    BI_local_set 3%N (*i*) ::
    BI_block (BT_valtype None) (
      BI_loop (BT_valtype None) (
        BI_local_get 3%N ::
        BI_const_num (Vi32 32) ::
        BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
        BI_testop T_i32 TO_eqz ::
        BI_br_if 1%N ::
        BI_local_get 0%N ::
        BI_local_get 3%N ::
        BI_binop T_i32 (Binop_i (BOI_shr SX_U)) ::
        BI_const_num (Vi32 1) ::
        BI_binop T_i32 (Binop_i BOI_and) ::
        BI_const_num (Vi32 1) ::
        BI_relop T_i32 (Relop_i ROI_eq) ::
        BI_if (BT_valtype None) (
          BI_local_get 1%N ::
          BI_local_get 2%N ::
          BI_binop T_i64 (Binop_i BOI_or) ::
          BI_local_set 1%N ::
          nil) (
          nil) ::
        BI_local_get 2%N ::
        BI_const_num (Vi64 1) ::
        BI_binop T_i64 (Binop_i BOI_shl) ::
        BI_local_set 2%N ::
        BI_local_get 3%N ::
        BI_const_num (Vi32 1) ::
        BI_local_tee 5%N ::
        BI_binop T_i32 (Binop_i BOI_add) ::
        BI_local_tee 6%N ::
        BI_local_get 5%N ::
        BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
        BI_if (BT_valtype None) (
          BI_unreachable ::
          nil) (
          nil) ::
        BI_local_get 6%N ::
        BI_local_set 3%N ::
        BI_br 0%N ::
        nil) ::
      nil) ::
    BI_local_get 1%N (*r*) ::
    BI_return ::
    BI_unreachable ::
    nil;
|}.

Definition model_mul_lo : module_func := {|
  modfunc_type := 12%N;
  modfunc_locals := T_num T_i32 :: T_num T_i32 :: nil;
  modfunc_body :=
    BI_global_get 0%N ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_local_tee 3%N (*__frame_ptr*) ::
    BI_global_set 0%N ::
    BI_local_get 3%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 0%N 3%N) ::
    BI_local_get 3%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 8%N 3%N) ::
    BI_local_get 3%N (*__frame_ptr*) ::
    BI_local_get 0%N (*a*) ::
    BI_local_get 1%N (*b*) ::
    BI_call 18%N ::
    BI_local_get 3%N (*__frame_ptr*) ::
    BI_local_set 2%N (*p*) ::
    BI_local_get 2%N (*p*) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 3%N (*__frame_ptr*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_global_set 0%N ::
    BI_return ::
    BI_unreachable ::
    nil;
|}.

Definition model_mul_hi : module_func := {|
  modfunc_type := 13%N;
  modfunc_locals := T_num T_i32 :: T_num T_i32 :: nil;
  modfunc_body :=
    BI_global_get 0%N ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_local_tee 3%N (*__frame_ptr*) ::
    BI_global_set 0%N ::
    BI_local_get 3%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 0%N 3%N) ::
    BI_local_get 3%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 8%N 3%N) ::
    BI_local_get 3%N (*__frame_ptr*) ::
    BI_local_get 0%N (*a*) ::
    BI_local_get 1%N (*b*) ::
    BI_call 18%N ::
    BI_local_get 3%N (*__frame_ptr*) ::
    BI_local_set 2%N (*p*) ::
    BI_local_get 2%N (*p*) ::
    BI_const_num (Vi32 4) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 3%N (*__frame_ptr*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_global_set 0%N ::
    BI_return ::
    BI_unreachable ::
    nil;
|}.

Definition model_mul_by_limb : module_func := {|
  modfunc_type := 14%N;
  modfunc_locals := T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil;
  modfunc_body :=
    BI_global_get 0%N ::
    BI_const_num (Vi32 32) ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_local_tee 6%N (*__frame_ptr*) ::
    BI_global_set 0%N ::
    BI_local_get 6%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 0%N 3%N) ::
    BI_local_get 6%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 8%N 3%N) ::
    BI_local_get 6%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 16%N 3%N) ::
    BI_local_get 6%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 24%N 3%N) ::
    BI_local_get 6%N (*__frame_ptr*) ::
    BI_local_get 1%N (*lo*) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 6%N (*__frame_ptr*) ::
    BI_const_num (Vi32 4) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 0%N (*hi*) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 6%N (*__frame_ptr*) ::
    BI_local_set 4%N (*n*) ::
    BI_local_get 6%N (*__frame_ptr*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 4%N (*n*) ::
    BI_local_get 2%N (*m*) ::
    BI_call 21%N ::
    BI_local_get 6%N (*__frame_ptr*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 5%N (*p*) ::
    BI_local_get 3%N (*k*) ::
    BI_const_num (Vi32 0) ::
    BI_relop T_i32 (Relop_i ROI_eq) ::
    BI_if (BT_valtype None) (
      BI_local_get 5%N ::
      BI_load T_i32 None (Ma 0%N 2%N) ::
      BI_local_get 6%N ::
      BI_const_num (Vi32 32) ::
      BI_binop T_i32 (Binop_i BOI_add) ::
      BI_global_set 0%N ::
      BI_return ::
      nil) (
      nil) ::
    BI_local_get 3%N (*k*) ::
    BI_const_num (Vi32 1) ::
    BI_relop T_i32 (Relop_i ROI_eq) ::
    BI_if (BT_valtype None) (
      BI_local_get 5%N ::
      BI_const_num (Vi32 4) ::
      BI_binop T_i32 (Binop_i BOI_add) ::
      BI_load T_i32 None (Ma 0%N 2%N) ::
      BI_local_get 6%N ::
      BI_const_num (Vi32 32) ::
      BI_binop T_i32 (Binop_i BOI_add) ::
      BI_global_set 0%N ::
      BI_return ::
      nil) (
      nil) ::
    BI_local_get 3%N (*k*) ::
    BI_const_num (Vi32 2) ::
    BI_relop T_i32 (Relop_i ROI_eq) ::
    BI_if (BT_valtype None) (
      BI_local_get 5%N ::
      BI_const_num (Vi32 8) ::
      BI_binop T_i32 (Binop_i BOI_add) ::
      BI_load T_i32 None (Ma 0%N 2%N) ::
      BI_local_get 6%N ::
      BI_const_num (Vi32 32) ::
      BI_binop T_i32 (Binop_i BOI_add) ::
      BI_global_set 0%N ::
      BI_return ::
      nil) (
      nil) ::
    BI_local_get 5%N (*p*) ::
    BI_const_num (Vi32 12) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 6%N (*__frame_ptr*) ::
    BI_const_num (Vi32 32) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_global_set 0%N ::
    BI_return ::
    BI_unreachable ::
    nil;
|}.

Definition model_div64 : module_func := {|
  modfunc_type := 15%N;
  modfunc_locals := T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil;
  modfunc_body :=
    BI_global_get 0%N ::
    BI_const_num (Vi32 32) ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_local_tee 5%N (*__frame_ptr*) ::
    BI_global_set 0%N ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 0%N 3%N) ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 8%N 3%N) ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 16%N 3%N) ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 24%N 3%N) ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_local_get 1%N (*lo*) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_const_num (Vi32 4) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 0%N (*hi*) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_local_set 3%N (*n*) ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 2%N (*d*) ::
    BI_call 17%N ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 4%N (*dd*) ::
    BI_local_get 3%N (*n*) ::
    BI_local_get 4%N (*dd*) ::
    BI_call 28%N ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_const_num (Vi32 32) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_global_set 0%N ::
    BI_return ::
    BI_unreachable ::
    nil;
|}.

Definition model_sqrt64 : module_func := {|
  modfunc_type := 16%N;
  modfunc_locals := T_num T_i32 :: T_num T_i32 :: nil;
  modfunc_body :=
    BI_global_get 0%N ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_local_tee 3%N (*__frame_ptr*) ::
    BI_global_set 0%N ::
    BI_local_get 3%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 0%N 3%N) ::
    BI_local_get 3%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 8%N 3%N) ::
    BI_local_get 3%N (*__frame_ptr*) ::
    BI_local_get 1%N (*lo*) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 3%N (*__frame_ptr*) ::
    BI_const_num (Vi32 4) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 0%N (*hi*) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 3%N (*__frame_ptr*) ::
    BI_local_set 2%N (*n*) ::
    BI_local_get 2%N (*n*) ::
    BI_call 29%N ::
    BI_local_get 3%N (*__frame_ptr*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_global_set 0%N ::
    BI_return ::
    BI_unreachable ::
    nil;
|}.

Definition wide_from_u32 : module_func := {|
  modfunc_type := 17%N;
  modfunc_locals := nil;
  modfunc_body :=
    BI_local_get 0%N (*sret*) ::
    BI_local_get 1%N (*x*) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 0%N (*sret*) ::
    BI_const_num (Vi32 4) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_const_num (Vi32 0) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 0%N (*sret*) ::
    BI_const_num (Vi32 8) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_const_num (Vi32 0) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 0%N (*sret*) ::
    BI_const_num (Vi32 12) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_const_num (Vi32 0) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_return ::
    BI_unreachable ::
    nil;
|}.

Definition wide_mul32 : module_func := {|
  modfunc_type := 18%N;
  modfunc_locals := T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil;
  modfunc_body :=
    BI_const_num (Vi32 65535) ::
    BI_local_set 3%N (*LOW16*) ::
    BI_local_get 1%N (*a*) ::
    BI_local_get 3%N (*LOW16*) ::
    BI_binop T_i32 (Binop_i BOI_and) ::
    BI_local_set 4%N (*al*) ::
    BI_local_get 1%N (*a*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i (BOI_shr SX_U)) ::
    BI_local_set 5%N (*ah*) ::
    BI_local_get 2%N (*b*) ::
    BI_local_get 3%N (*LOW16*) ::
    BI_binop T_i32 (Binop_i BOI_and) ::
    BI_local_set 6%N (*bl*) ::
    BI_local_get 2%N (*b*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i (BOI_shr SX_U)) ::
    BI_local_set 7%N (*bh*) ::
    BI_local_get 4%N (*al*) ::
    BI_local_get 6%N (*bl*) ::
    BI_local_set 17%N ::
    BI_local_tee 16%N ::
    BI_local_get 17%N ::
    BI_binop T_i32 (Binop_i BOI_mul) ::
    BI_local_set 18%N ::
    BI_local_get 17%N ::
    BI_const_num (Vi32 0) ::
    BI_relop T_i32 (Relop_i ROI_ne) ::
    BI_if (BT_valtype None) (
      BI_local_get 18%N ::
      BI_local_get 17%N ::
      BI_binop T_i32 (Binop_i (BOI_div SX_U)) ::
      BI_local_get 16%N ::
      BI_relop T_i32 (Relop_i ROI_ne) ::
      BI_if (BT_valtype None) (
        BI_unreachable ::
        nil) (
        nil) ::
      nil) (
      nil) ::
    BI_local_get 18%N ::
    BI_local_set 8%N (*ll*) ::
    BI_local_get 4%N (*al*) ::
    BI_local_get 7%N (*bh*) ::
    BI_local_set 17%N ::
    BI_local_tee 16%N ::
    BI_local_get 17%N ::
    BI_binop T_i32 (Binop_i BOI_mul) ::
    BI_local_set 18%N ::
    BI_local_get 17%N ::
    BI_const_num (Vi32 0) ::
    BI_relop T_i32 (Relop_i ROI_ne) ::
    BI_if (BT_valtype None) (
      BI_local_get 18%N ::
      BI_local_get 17%N ::
      BI_binop T_i32 (Binop_i (BOI_div SX_U)) ::
      BI_local_get 16%N ::
      BI_relop T_i32 (Relop_i ROI_ne) ::
      BI_if (BT_valtype None) (
        BI_unreachable ::
        nil) (
        nil) ::
      nil) (
      nil) ::
    BI_local_get 18%N ::
    BI_local_set 9%N (*lh*) ::
    BI_local_get 5%N (*ah*) ::
    BI_local_get 6%N (*bl*) ::
    BI_local_set 17%N ::
    BI_local_tee 16%N ::
    BI_local_get 17%N ::
    BI_binop T_i32 (Binop_i BOI_mul) ::
    BI_local_set 18%N ::
    BI_local_get 17%N ::
    BI_const_num (Vi32 0) ::
    BI_relop T_i32 (Relop_i ROI_ne) ::
    BI_if (BT_valtype None) (
      BI_local_get 18%N ::
      BI_local_get 17%N ::
      BI_binop T_i32 (Binop_i (BOI_div SX_U)) ::
      BI_local_get 16%N ::
      BI_relop T_i32 (Relop_i ROI_ne) ::
      BI_if (BT_valtype None) (
        BI_unreachable ::
        nil) (
        nil) ::
      nil) (
      nil) ::
    BI_local_get 18%N ::
    BI_local_set 10%N (*hl*) ::
    BI_local_get 5%N (*ah*) ::
    BI_local_get 7%N (*bh*) ::
    BI_local_set 17%N ::
    BI_local_tee 16%N ::
    BI_local_get 17%N ::
    BI_binop T_i32 (Binop_i BOI_mul) ::
    BI_local_set 18%N ::
    BI_local_get 17%N ::
    BI_const_num (Vi32 0) ::
    BI_relop T_i32 (Relop_i ROI_ne) ::
    BI_if (BT_valtype None) (
      BI_local_get 18%N ::
      BI_local_get 17%N ::
      BI_binop T_i32 (Binop_i (BOI_div SX_U)) ::
      BI_local_get 16%N ::
      BI_relop T_i32 (Relop_i ROI_ne) ::
      BI_if (BT_valtype None) (
        BI_unreachable ::
        nil) (
        nil) ::
      nil) (
      nil) ::
    BI_local_get 18%N ::
    BI_local_set 11%N (*hh*) ::
    BI_local_get 9%N (*lh*) ::
    BI_local_get 8%N (*ll*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i (BOI_shr SX_U)) ::
    BI_local_tee 17%N ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_tee 18%N ::
    BI_local_get 17%N ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 18%N ::
    BI_local_set 12%N (*mid1*) ::
    BI_local_get 10%N (*hl*) ::
    BI_local_get 12%N (*mid1*) ::
    BI_local_get 3%N (*LOW16*) ::
    BI_binop T_i32 (Binop_i BOI_and) ::
    BI_local_tee 17%N ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_tee 18%N ::
    BI_local_get 17%N ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 18%N ::
    BI_local_set 13%N (*mid2*) ::
    BI_local_get 13%N (*mid2*) ::
    BI_local_get 3%N (*LOW16*) ::
    BI_binop T_i32 (Binop_i BOI_and) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_shl) ::
    BI_local_get 8%N (*ll*) ::
    BI_local_get 3%N (*LOW16*) ::
    BI_binop T_i32 (Binop_i BOI_and) ::
    BI_binop T_i32 (Binop_i BOI_or) ::
    BI_local_set 14%N (*lo*) ::
    BI_local_get 11%N (*hh*) ::
    BI_local_get 12%N (*mid1*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i (BOI_shr SX_U)) ::
    BI_local_tee 17%N ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_tee 18%N ::
    BI_local_get 17%N ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 18%N ::
    BI_local_get 13%N (*mid2*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i (BOI_shr SX_U)) ::
    BI_local_tee 17%N ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_tee 18%N ::
    BI_local_get 17%N ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 18%N ::
    BI_local_set 15%N (*hi*) ::
    BI_local_get 0%N (*sret*) ::
    BI_local_get 14%N (*lo*) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 0%N (*sret*) ::
    BI_const_num (Vi32 4) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 15%N (*hi*) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 0%N (*sret*) ::
    BI_const_num (Vi32 8) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_const_num (Vi32 0) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 0%N (*sret*) ::
    BI_const_num (Vi32 12) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_const_num (Vi32 0) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_return ::
    BI_unreachable ::
    nil;
|}.

Definition wide_add : module_func := {|
  modfunc_type := 19%N;
  modfunc_locals := T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil;
  modfunc_body :=
    BI_local_get 1%N (*a*) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 2%N (*b*) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 3%N (*s0*) ::
    BI_const_num (Vi32 0) ::
    BI_local_set 4%N (*c0*) ::
    BI_local_get 3%N (*s0*) ::
    BI_local_get 1%N (*a*) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_if (BT_valtype None) (
      BI_const_num (Vi32 1) ::
      BI_local_set 4%N ::
      nil) (
      nil) ::
    BI_local_get 1%N (*a*) ::
    BI_const_num (Vi32 4) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 2%N (*b*) ::
    BI_const_num (Vi32 4) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 5%N (*t1*) ::
    BI_local_get 5%N (*t1*) ::
    BI_local_get 4%N (*c0*) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 6%N (*s1*) ::
    BI_const_num (Vi32 0) ::
    BI_local_set 7%N (*c1*) ::
    BI_local_get 5%N (*t1*) ::
    BI_local_get 1%N (*a*) ::
    BI_const_num (Vi32 4) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_if (BT_valtype None) (
      BI_const_num (Vi32 1) ::
      BI_local_set 7%N ::
      nil) (
      nil) ::
    BI_local_get 6%N (*s1*) ::
    BI_local_get 5%N (*t1*) ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_if (BT_valtype None) (
      BI_const_num (Vi32 1) ::
      BI_local_set 7%N ::
      nil) (
      nil) ::
    BI_local_get 1%N (*a*) ::
    BI_const_num (Vi32 8) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 2%N (*b*) ::
    BI_const_num (Vi32 8) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 8%N (*t2*) ::
    BI_local_get 8%N (*t2*) ::
    BI_local_get 7%N (*c1*) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 9%N (*s2*) ::
    BI_const_num (Vi32 0) ::
    BI_local_set 10%N (*c2*) ::
    BI_local_get 8%N (*t2*) ::
    BI_local_get 1%N (*a*) ::
    BI_const_num (Vi32 8) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_if (BT_valtype None) (
      BI_const_num (Vi32 1) ::
      BI_local_set 10%N ::
      nil) (
      nil) ::
    BI_local_get 9%N (*s2*) ::
    BI_local_get 8%N (*t2*) ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_if (BT_valtype None) (
      BI_const_num (Vi32 1) ::
      BI_local_set 10%N ::
      nil) (
      nil) ::
    BI_local_get 1%N (*a*) ::
    BI_const_num (Vi32 12) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 2%N (*b*) ::
    BI_const_num (Vi32 12) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_local_tee 13%N ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_tee 14%N ::
    BI_local_get 13%N ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 14%N ::
    BI_local_get 10%N (*c2*) ::
    BI_local_tee 13%N ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_tee 14%N ::
    BI_local_get 13%N ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 14%N ::
    BI_local_set 11%N (*s3*) ::
    BI_local_get 0%N (*sret*) ::
    BI_local_get 3%N (*s0*) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 0%N (*sret*) ::
    BI_const_num (Vi32 4) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 6%N (*s1*) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 0%N (*sret*) ::
    BI_const_num (Vi32 8) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 9%N (*s2*) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 0%N (*sret*) ::
    BI_const_num (Vi32 12) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 11%N (*s3*) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_return ::
    BI_unreachable ::
    nil;
|}.

Definition wide_sub : module_func := {|
  modfunc_type := 20%N;
  modfunc_locals := T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil;
  modfunc_body :=
    BI_local_get 1%N (*a*) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 2%N (*b*) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_local_set 3%N (*d0*) ::
    BI_const_num (Vi32 0) ::
    BI_local_set 4%N (*br0*) ::
    BI_local_get 1%N (*a*) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 2%N (*b*) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_if (BT_valtype None) (
      BI_const_num (Vi32 1) ::
      BI_local_set 4%N ::
      nil) (
      nil) ::
    BI_local_get 1%N (*a*) ::
    BI_const_num (Vi32 4) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 2%N (*b*) ::
    BI_const_num (Vi32 4) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_local_set 5%N (*t1*) ::
    BI_local_get 5%N (*t1*) ::
    BI_local_get 4%N (*br0*) ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_local_set 6%N (*d1*) ::
    BI_const_num (Vi32 0) ::
    BI_local_set 7%N (*br1*) ::
    BI_local_get 1%N (*a*) ::
    BI_const_num (Vi32 4) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 2%N (*b*) ::
    BI_const_num (Vi32 4) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_if (BT_valtype None) (
      BI_const_num (Vi32 1) ::
      BI_local_set 7%N ::
      nil) (
      nil) ::
    BI_local_get 5%N (*t1*) ::
    BI_local_get 4%N (*br0*) ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_if (BT_valtype None) (
      BI_const_num (Vi32 1) ::
      BI_local_set 7%N ::
      nil) (
      nil) ::
    BI_local_get 1%N (*a*) ::
    BI_const_num (Vi32 8) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 2%N (*b*) ::
    BI_const_num (Vi32 8) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_local_set 8%N (*t2*) ::
    BI_local_get 8%N (*t2*) ::
    BI_local_get 7%N (*br1*) ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_local_set 9%N (*d2*) ::
    BI_const_num (Vi32 0) ::
    BI_local_set 10%N (*br2*) ::
    BI_local_get 1%N (*a*) ::
    BI_const_num (Vi32 8) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 2%N (*b*) ::
    BI_const_num (Vi32 8) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_if (BT_valtype None) (
      BI_const_num (Vi32 1) ::
      BI_local_set 10%N ::
      nil) (
      nil) ::
    BI_local_get 8%N (*t2*) ::
    BI_local_get 7%N (*br1*) ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_if (BT_valtype None) (
      BI_const_num (Vi32 1) ::
      BI_local_set 10%N ::
      nil) (
      nil) ::
    BI_local_get 1%N (*a*) ::
    BI_const_num (Vi32 12) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 2%N (*b*) ::
    BI_const_num (Vi32 12) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_local_set 13%N ::
    BI_local_tee 12%N ::
    BI_local_get 13%N ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 12%N ::
    BI_local_get 13%N ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_local_get 10%N (*br2*) ::
    BI_local_set 13%N ::
    BI_local_tee 12%N ::
    BI_local_get 13%N ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 12%N ::
    BI_local_get 13%N ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_local_set 11%N (*d3*) ::
    BI_local_get 0%N (*sret*) ::
    BI_local_get 3%N (*d0*) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 0%N (*sret*) ::
    BI_const_num (Vi32 4) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 6%N (*d1*) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 0%N (*sret*) ::
    BI_const_num (Vi32 8) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 9%N (*d2*) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 0%N (*sret*) ::
    BI_const_num (Vi32 12) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 11%N (*d3*) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_return ::
    BI_unreachable ::
    nil;
|}.

Definition wide_mul_by : module_func := {|
  modfunc_type := 21%N;
  modfunc_locals := T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil;
  modfunc_body :=
    BI_global_get 0%N ::
    BI_const_num (Vi32 160) ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_local_tee 13%N (*__frame_ptr*) ::
    BI_global_set 0%N ::
    BI_const_num (Vi32 0) ::
    BI_local_set 14%N ::
    BI_loop (BT_valtype None) (
      BI_local_get 13%N ::
      BI_local_get 14%N ::
      BI_binop T_i32 (Binop_i BOI_add) ::
      BI_const_num (Vi64 0) ::
      BI_store T_i64 None (Ma 0%N 3%N) ::
      BI_local_get 13%N ::
      BI_local_get 14%N ::
      BI_binop T_i32 (Binop_i BOI_add) ::
      BI_const_num (Vi64 0) ::
      BI_store T_i64 None (Ma 8%N 3%N) ::
      BI_local_get 14%N ::
      BI_const_num (Vi32 16) ::
      BI_binop T_i32 (Binop_i BOI_add) ::
      BI_local_tee 14%N ::
      BI_const_num (Vi32 160) ::
      BI_relop T_i32 (Relop_i ROI_ne) ::
      BI_br_if 0%N ::
      nil) ::
    BI_local_get 13%N (*__frame_ptr*) ::
    BI_local_get 1%N (*a*) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 2%N (*m*) ::
    BI_call 18%N ::
    BI_local_get 13%N (*__frame_ptr*) ::
    BI_local_set 3%N (*p0*) ::
    BI_local_get 13%N (*__frame_ptr*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 1%N (*a*) ::
    BI_const_num (Vi32 4) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 2%N (*m*) ::
    BI_call 18%N ::
    BI_local_get 13%N (*__frame_ptr*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 4%N (*p1*) ::
    BI_local_get 13%N (*__frame_ptr*) ::
    BI_const_num (Vi32 32) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 1%N (*a*) ::
    BI_const_num (Vi32 8) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 2%N (*m*) ::
    BI_call 18%N ::
    BI_local_get 13%N (*__frame_ptr*) ::
    BI_const_num (Vi32 32) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 5%N (*p2*) ::
    BI_local_get 13%N (*__frame_ptr*) ::
    BI_const_num (Vi32 48) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 1%N (*a*) ::
    BI_const_num (Vi32 12) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 2%N (*m*) ::
    BI_call 18%N ::
    BI_local_get 13%N (*__frame_ptr*) ::
    BI_const_num (Vi32 48) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 6%N (*p3*) ::
    BI_local_get 6%N (*p3*) ::
    BI_const_num (Vi32 4) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_const_num (Vi32 0) ::
    BI_relop T_i32 (Relop_i ROI_eq) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 13%N (*__frame_ptr*) ::
    BI_const_num (Vi32 64) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 3%N (*p0*) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 13%N (*__frame_ptr*) ::
    BI_const_num (Vi32 68) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 3%N (*p0*) ::
    BI_const_num (Vi32 4) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 13%N (*__frame_ptr*) ::
    BI_const_num (Vi32 64) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 7%N (*x0*) ::
    BI_local_get 13%N (*__frame_ptr*) ::
    BI_const_num (Vi32 84) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 4%N (*p1*) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 13%N (*__frame_ptr*) ::
    BI_const_num (Vi32 88) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 4%N (*p1*) ::
    BI_const_num (Vi32 4) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 13%N (*__frame_ptr*) ::
    BI_const_num (Vi32 80) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 8%N (*x1*) ::
    BI_local_get 13%N (*__frame_ptr*) ::
    BI_const_num (Vi32 104) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 5%N (*p2*) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 13%N (*__frame_ptr*) ::
    BI_const_num (Vi32 108) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 5%N (*p2*) ::
    BI_const_num (Vi32 4) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 13%N (*__frame_ptr*) ::
    BI_const_num (Vi32 96) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 9%N (*x2*) ::
    BI_local_get 13%N (*__frame_ptr*) ::
    BI_const_num (Vi32 124) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 6%N (*p3*) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 13%N (*__frame_ptr*) ::
    BI_const_num (Vi32 112) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 10%N (*x3*) ::
    BI_local_get 13%N (*__frame_ptr*) ::
    BI_const_num (Vi32 128) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 7%N (*x0*) ::
    BI_local_get 8%N (*x1*) ::
    BI_call 19%N ::
    BI_local_get 13%N (*__frame_ptr*) ::
    BI_const_num (Vi32 128) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 11%N (*s01*) ::
    BI_local_get 13%N (*__frame_ptr*) ::
    BI_const_num (Vi32 144) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 11%N (*s01*) ::
    BI_local_get 9%N (*x2*) ::
    BI_call 19%N ::
    BI_local_get 13%N (*__frame_ptr*) ::
    BI_const_num (Vi32 144) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_set 12%N (*s012*) ::
    BI_local_get 0%N (*sret*) ::
    BI_local_get 12%N (*s012*) ::
    BI_local_get 10%N (*x3*) ::
    BI_call 19%N ::
    BI_local_get 13%N (*__frame_ptr*) ::
    BI_const_num (Vi32 160) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_global_set 0%N ::
    BI_return ::
    BI_unreachable ::
    nil;
|}.

Definition wide_eq : module_func := {|
  modfunc_type := 22%N;
  modfunc_locals := nil;
  modfunc_body :=
    BI_local_get 0%N (*a*) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 1%N (*b*) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_relop T_i32 (Relop_i ROI_eq) ::
    BI_if (BT_valtype (Some (T_num T_i32))) (
      BI_local_get 0%N ::
      BI_const_num (Vi32 4) ::
      BI_binop T_i32 (Binop_i BOI_add) ::
      BI_load T_i32 None (Ma 0%N 2%N) ::
      BI_local_get 1%N ::
      BI_const_num (Vi32 4) ::
      BI_binop T_i32 (Binop_i BOI_add) ::
      BI_load T_i32 None (Ma 0%N 2%N) ::
      BI_relop T_i32 (Relop_i ROI_eq) ::
      nil) (
      BI_const_num (Vi32 0) ::
      nil) ::
    BI_if (BT_valtype (Some (T_num T_i32))) (
      BI_local_get 0%N ::
      BI_const_num (Vi32 8) ::
      BI_binop T_i32 (Binop_i BOI_add) ::
      BI_load T_i32 None (Ma 0%N 2%N) ::
      BI_local_get 1%N ::
      BI_const_num (Vi32 8) ::
      BI_binop T_i32 (Binop_i BOI_add) ::
      BI_load T_i32 None (Ma 0%N 2%N) ::
      BI_relop T_i32 (Relop_i ROI_eq) ::
      nil) (
      BI_const_num (Vi32 0) ::
      nil) ::
    BI_if (BT_valtype (Some (T_num T_i32))) (
      BI_local_get 0%N ::
      BI_const_num (Vi32 12) ::
      BI_binop T_i32 (Binop_i BOI_add) ::
      BI_load T_i32 None (Ma 0%N 2%N) ::
      BI_local_get 1%N ::
      BI_const_num (Vi32 12) ::
      BI_binop T_i32 (Binop_i BOI_add) ::
      BI_load T_i32 None (Ma 0%N 2%N) ::
      BI_relop T_i32 (Relop_i ROI_eq) ::
      nil) (
      BI_const_num (Vi32 0) ::
      nil) ::
    BI_return ::
    BI_unreachable ::
    nil;
|}.

Definition wide_lt : module_func := {|
  modfunc_type := 23%N;
  modfunc_locals := nil;
  modfunc_body :=
    BI_local_get 0%N (*a*) ::
    BI_const_num (Vi32 12) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 1%N (*b*) ::
    BI_const_num (Vi32 12) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_relop T_i32 (Relop_i ROI_ne) ::
    BI_if (BT_valtype None) (
      BI_local_get 0%N ::
      BI_const_num (Vi32 12) ::
      BI_binop T_i32 (Binop_i BOI_add) ::
      BI_load T_i32 None (Ma 0%N 2%N) ::
      BI_local_get 1%N ::
      BI_const_num (Vi32 12) ::
      BI_binop T_i32 (Binop_i BOI_add) ::
      BI_load T_i32 None (Ma 0%N 2%N) ::
      BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
      BI_return ::
      nil) (
      nil) ::
    BI_local_get 0%N (*a*) ::
    BI_const_num (Vi32 8) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 1%N (*b*) ::
    BI_const_num (Vi32 8) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_relop T_i32 (Relop_i ROI_ne) ::
    BI_if (BT_valtype None) (
      BI_local_get 0%N ::
      BI_const_num (Vi32 8) ::
      BI_binop T_i32 (Binop_i BOI_add) ::
      BI_load T_i32 None (Ma 0%N 2%N) ::
      BI_local_get 1%N ::
      BI_const_num (Vi32 8) ::
      BI_binop T_i32 (Binop_i BOI_add) ::
      BI_load T_i32 None (Ma 0%N 2%N) ::
      BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
      BI_return ::
      nil) (
      nil) ::
    BI_local_get 0%N (*a*) ::
    BI_const_num (Vi32 4) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 1%N (*b*) ::
    BI_const_num (Vi32 4) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_relop T_i32 (Relop_i ROI_ne) ::
    BI_if (BT_valtype None) (
      BI_local_get 0%N ::
      BI_const_num (Vi32 4) ::
      BI_binop T_i32 (Binop_i BOI_add) ::
      BI_load T_i32 None (Ma 0%N 2%N) ::
      BI_local_get 1%N ::
      BI_const_num (Vi32 4) ::
      BI_binop T_i32 (Binop_i BOI_add) ::
      BI_load T_i32 None (Ma 0%N 2%N) ::
      BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
      BI_return ::
      nil) (
      nil) ::
    BI_local_get 0%N (*a*) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 1%N (*b*) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
    BI_return ::
    BI_unreachable ::
    nil;
|}.

Definition wide_le : module_func := {|
  modfunc_type := 24%N;
  modfunc_locals := nil;
  modfunc_body :=
    BI_local_get 1%N (*b*) ::
    BI_local_get 0%N (*a*) ::
    BI_call 23%N ::
    BI_testop T_i32 TO_eqz ::
    BI_return ::
    BI_unreachable ::
    nil;
|}.

Definition wide_shr32 : module_func := {|
  modfunc_type := 25%N;
  modfunc_locals := nil;
  modfunc_body :=
    BI_local_get 0%N (*sret*) ::
    BI_local_get 1%N (*a*) ::
    BI_const_num (Vi32 4) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 0%N (*sret*) ::
    BI_const_num (Vi32 4) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 1%N (*a*) ::
    BI_const_num (Vi32 8) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 0%N (*sret*) ::
    BI_const_num (Vi32 8) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 1%N (*a*) ::
    BI_const_num (Vi32 12) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 0%N (*sret*) ::
    BI_const_num (Vi32 12) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_const_num (Vi32 0) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_return ::
    BI_unreachable ::
    nil;
|}.

Definition wide_shl1_or : module_func := {|
  modfunc_type := 26%N;
  modfunc_locals := T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil;
  modfunc_body :=
    BI_local_get 1%N (*a*) ::
    BI_const_num (Vi32 12) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_const_num (Vi32 31) ::
    BI_binop T_i32 (Binop_i (BOI_shr SX_U)) ::
    BI_const_num (Vi32 0) ::
    BI_relop T_i32 (Relop_i ROI_eq) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 1%N (*a*) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_const_num (Vi32 1) ::
    BI_binop T_i32 (Binop_i BOI_shl) ::
    BI_local_get 2%N (*bit*) ::
    BI_binop T_i32 (Binop_i BOI_or) ::
    BI_local_set 3%N (*n0*) ::
    BI_local_get 1%N (*a*) ::
    BI_const_num (Vi32 4) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_const_num (Vi32 1) ::
    BI_binop T_i32 (Binop_i BOI_shl) ::
    BI_local_get 1%N (*a*) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_const_num (Vi32 31) ::
    BI_binop T_i32 (Binop_i (BOI_shr SX_U)) ::
    BI_binop T_i32 (Binop_i BOI_or) ::
    BI_local_set 4%N (*n1*) ::
    BI_local_get 1%N (*a*) ::
    BI_const_num (Vi32 8) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_const_num (Vi32 1) ::
    BI_binop T_i32 (Binop_i BOI_shl) ::
    BI_local_get 1%N (*a*) ::
    BI_const_num (Vi32 4) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_const_num (Vi32 31) ::
    BI_binop T_i32 (Binop_i (BOI_shr SX_U)) ::
    BI_binop T_i32 (Binop_i BOI_or) ::
    BI_local_set 5%N (*n2*) ::
    BI_local_get 1%N (*a*) ::
    BI_const_num (Vi32 12) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_const_num (Vi32 1) ::
    BI_binop T_i32 (Binop_i BOI_shl) ::
    BI_local_get 1%N (*a*) ::
    BI_const_num (Vi32 8) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_const_num (Vi32 31) ::
    BI_binop T_i32 (Binop_i (BOI_shr SX_U)) ::
    BI_binop T_i32 (Binop_i BOI_or) ::
    BI_local_set 6%N (*n3*) ::
    BI_local_get 0%N (*sret*) ::
    BI_local_get 3%N (*n0*) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 0%N (*sret*) ::
    BI_const_num (Vi32 4) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 4%N (*n1*) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 0%N (*sret*) ::
    BI_const_num (Vi32 8) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 5%N (*n2*) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_local_get 0%N (*sret*) ::
    BI_const_num (Vi32 12) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_local_get 6%N (*n3*) ::
    BI_store T_i32 None (Ma 0%N 2%N) ::
    BI_return ::
    BI_unreachable ::
    nil;
|}.

Definition wide_quotient_fits : module_func := {|
  modfunc_type := 27%N;
  modfunc_locals := T_num T_i32 :: T_num T_i32 :: nil;
  modfunc_body :=
    BI_global_get 0%N ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_local_tee 3%N (*__frame_ptr*) ::
    BI_global_set 0%N ::
    BI_local_get 3%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 0%N 3%N) ::
    BI_local_get 3%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 8%N 3%N) ::
    BI_local_get 3%N (*__frame_ptr*) ::
    BI_local_get 0%N (*n*) ::
    BI_call 25%N ::
    BI_local_get 3%N (*__frame_ptr*) ::
    BI_local_set 2%N (*h*) ::
    BI_local_get 2%N (*h*) ::
    BI_local_get 1%N (*d*) ::
    BI_call 23%N ::
    BI_local_get 3%N (*__frame_ptr*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_global_set 0%N ::
    BI_return ::
    BI_unreachable ::
    nil;
|}.

Definition wide_div_to_u32 : module_func := {|
  modfunc_type := 28%N;
  modfunc_locals := T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil;
  modfunc_body :=
    BI_global_get 0%N ::
    BI_const_num (Vi32 48) ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_local_tee 8%N (*__frame_ptr*) ::
    BI_global_set 0%N ::
    BI_local_get 8%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 0%N 3%N) ::
    BI_local_get 8%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 8%N 3%N) ::
    BI_local_get 8%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 16%N 3%N) ::
    BI_local_get 8%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 24%N 3%N) ::
    BI_local_get 8%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 32%N 3%N) ::
    BI_local_get 8%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 40%N 3%N) ::
    BI_local_get 0%N (*num*) ::
    BI_local_get 1%N (*den*) ::
    BI_call 27%N ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_local_get 8%N (*__frame_ptr*) ::
    BI_local_get 0%N (*num*) ::
    BI_call 25%N ::
    BI_local_get 8%N (*__frame_ptr*) ::
    BI_local_set 2%N (*r*) ::
    BI_const_num (Vi32 0) ::
    BI_local_set 3%N (*q*) ::
    BI_const_num (Vi32 0) ::
    BI_local_set 4%N (*i*) ::
    BI_block (BT_valtype None) (
      BI_loop (BT_valtype None) (
        BI_local_get 4%N ::
        BI_const_num (Vi32 32) ::
        BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
        BI_testop T_i32 TO_eqz ::
        BI_br_if 1%N ::
        BI_local_get 0%N ::
        BI_load T_i32 None (Ma 0%N 2%N) ::
        BI_const_num (Vi32 31) ::
        BI_local_get 4%N ::
        BI_local_set 10%N ::
        BI_local_tee 9%N ::
        BI_local_get 10%N ::
        BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
        BI_if (BT_valtype None) (
          BI_unreachable ::
          nil) (
          nil) ::
        BI_local_get 9%N ::
        BI_local_get 10%N ::
        BI_binop T_i32 (Binop_i BOI_sub) ::
        BI_binop T_i32 (Binop_i (BOI_shr SX_U)) ::
        BI_const_num (Vi32 1) ::
        BI_binop T_i32 (Binop_i BOI_and) ::
        BI_local_set 5%N ::
        BI_local_get 8%N ::
        BI_const_num (Vi32 16) ::
        BI_binop T_i32 (Binop_i BOI_add) ::
        BI_local_get 2%N ::
        BI_local_get 5%N ::
        BI_call 26%N ::
        BI_local_get 8%N ::
        BI_const_num (Vi32 16) ::
        BI_binop T_i32 (Binop_i BOI_add) ::
        BI_local_set 6%N ::
        BI_local_get 3%N ::
        BI_const_num (Vi32 1) ::
        BI_binop T_i32 (Binop_i BOI_shl) ::
        BI_local_set 3%N ::
        BI_local_get 1%N ::
        BI_local_get 6%N ::
        BI_call 24%N ::
        BI_if (BT_valtype None) (
          BI_local_get 8%N ::
          BI_const_num (Vi32 32) ::
          BI_binop T_i32 (Binop_i BOI_add) ::
          BI_local_get 6%N ::
          BI_local_get 1%N ::
          BI_call 20%N ::
          BI_local_get 8%N ::
          BI_const_num (Vi32 32) ::
          BI_binop T_i32 (Binop_i BOI_add) ::
          BI_local_set 7%N ::
          BI_local_get 2%N ::
          BI_local_get 7%N ::
          BI_local_set 13%N ::
          BI_local_set 12%N ::
          BI_local_get 12%N ::
          BI_local_get 13%N ::
          BI_load T_i64 None (Ma 0%N 0%N) ::
          BI_store T_i64 None (Ma 0%N 0%N) ::
          BI_local_get 12%N ::
          BI_local_get 13%N ::
          BI_load T_i64 None (Ma 8%N 0%N) ::
          BI_store T_i64 None (Ma 8%N 0%N) ::
          BI_local_get 3%N ::
          BI_const_num (Vi32 1) ::
          BI_binop T_i32 (Binop_i BOI_or) ::
          BI_local_set 3%N ::
          nil) (
          BI_local_get 2%N ::
          BI_local_get 6%N ::
          BI_local_set 13%N ::
          BI_local_set 12%N ::
          BI_local_get 12%N ::
          BI_local_get 13%N ::
          BI_load T_i64 None (Ma 0%N 0%N) ::
          BI_store T_i64 None (Ma 0%N 0%N) ::
          BI_local_get 12%N ::
          BI_local_get 13%N ::
          BI_load T_i64 None (Ma 8%N 0%N) ::
          BI_store T_i64 None (Ma 8%N 0%N) ::
          nil) ::
        BI_local_get 4%N ::
        BI_const_num (Vi32 1) ::
        BI_local_tee 10%N ::
        BI_binop T_i32 (Binop_i BOI_add) ::
        BI_local_tee 11%N ::
        BI_local_get 10%N ::
        BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
        BI_if (BT_valtype None) (
          BI_unreachable ::
          nil) (
          nil) ::
        BI_local_get 11%N ::
        BI_local_set 4%N ::
        BI_br 0%N ::
        nil) ::
      nil) ::
    BI_local_get 3%N (*q*) ::
    BI_local_get 8%N (*__frame_ptr*) ::
    BI_const_num (Vi32 48) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_global_set 0%N ::
    BI_return ::
    BI_unreachable ::
    nil;
|}.

Definition wide_isqrt : module_func := {|
  modfunc_type := 29%N;
  modfunc_locals := T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil;
  modfunc_body :=
    BI_global_get 0%N ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_sub) ::
    BI_local_tee 5%N (*__frame_ptr*) ::
    BI_global_set 0%N ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 0%N 3%N) ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_const_num (Vi64 0) ::
    BI_store T_i64 None (Ma 8%N 3%N) ::
    BI_local_get 0%N (*n*) ::
    BI_const_num (Vi32 8) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_load T_i32 None (Ma 0%N 2%N) ::
    BI_const_num (Vi32 0) ::
    BI_relop T_i32 (Relop_i ROI_eq) ::
    BI_if (BT_valtype (Some (T_num T_i32))) (
      BI_local_get 0%N ::
      BI_const_num (Vi32 12) ::
      BI_binop T_i32 (Binop_i BOI_add) ::
      BI_load T_i32 None (Ma 0%N 2%N) ::
      BI_const_num (Vi32 0) ::
      BI_relop T_i32 (Relop_i ROI_eq) ::
      nil) (
      BI_const_num (Vi32 0) ::
      nil) ::
    BI_testop T_i32 TO_eqz ::
    BI_if (BT_valtype None) (
      BI_unreachable ::
      nil) (
      nil) ::
    BI_const_num (Vi32 0) ::
    BI_local_set 1%N (*r*) ::
    BI_const_num (Vi32 0) ::
    BI_local_set 2%N (*i*) ::
    BI_block (BT_valtype None) (
      BI_loop (BT_valtype None) (
        BI_local_get 2%N ::
        BI_const_num (Vi32 32) ::
        BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
        BI_testop T_i32 TO_eqz ::
        BI_br_if 1%N ::
        BI_local_get 1%N ::
        BI_const_num (Vi32 1) ::
        BI_const_num (Vi32 31) ::
        BI_local_get 2%N ::
        BI_local_set 7%N ::
        BI_local_tee 6%N ::
        BI_local_get 7%N ::
        BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
        BI_if (BT_valtype None) (
          BI_unreachable ::
          nil) (
          nil) ::
        BI_local_get 6%N ::
        BI_local_get 7%N ::
        BI_binop T_i32 (Binop_i BOI_sub) ::
        BI_binop T_i32 (Binop_i BOI_shl) ::
        BI_binop T_i32 (Binop_i BOI_or) ::
        BI_local_set 3%N ::
        BI_local_get 5%N ::
        BI_local_get 3%N ::
        BI_local_get 3%N ::
        BI_call 18%N ::
        BI_local_get 5%N ::
        BI_local_set 4%N ::
        BI_local_get 4%N ::
        BI_local_get 0%N ::
        BI_call 24%N ::
        BI_if (BT_valtype None) (
          BI_local_get 3%N ::
          BI_local_set 1%N ::
          nil) (
          nil) ::
        BI_local_get 2%N ::
        BI_const_num (Vi32 1) ::
        BI_local_tee 7%N ::
        BI_binop T_i32 (Binop_i BOI_add) ::
        BI_local_tee 8%N ::
        BI_local_get 7%N ::
        BI_relop T_i32 (Relop_i (ROI_lt SX_U)) ::
        BI_if (BT_valtype None) (
          BI_unreachable ::
          nil) (
          nil) ::
        BI_local_get 8%N ::
        BI_local_set 2%N ::
        BI_br 0%N ::
        nil) ::
      nil) ::
    BI_local_get 1%N (*r*) ::
    BI_local_get 5%N (*__frame_ptr*) ::
    BI_const_num (Vi32 16) ::
    BI_binop T_i32 (Binop_i BOI_add) ::
    BI_global_set 0%N ::
    BI_return ::
    BI_unreachable ::
    nil;
|}.

Definition main : module := {|
  mod_types :=
    Tf (nil) (T_num T_i32 :: nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (T_num T_i32 :: nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (T_num T_i32 :: nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (T_num T_i32 :: nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: nil) (T_num T_i32 :: nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (T_num T_i32 :: nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (T_num T_i32 :: nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (T_num T_i32 :: nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (T_num T_i32 :: nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (T_num T_i32 :: nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (T_num T_i32 :: nil) ::
    Tf (T_num T_i32 :: nil) (T_num T_i64 :: nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: nil) (T_num T_i32 :: nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: nil) (T_num T_i32 :: nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (T_num T_i32 :: nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (T_num T_i32 :: nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: nil) (T_num T_i32 :: nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: nil) (T_num T_i32 :: nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: nil) (T_num T_i32 :: nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: nil) (T_num T_i32 :: nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: nil) (T_num T_i32 :: nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: nil) (T_num T_i32 :: nil) ::
    Tf (T_num T_i32 :: nil) (T_num T_i32 :: nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (nil) (nil) ::
    Tf (nil) (nil) ::
    Tf (T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    Tf (T_num T_i32 :: T_num T_i32 :: nil) (nil) ::
    nil;
  mod_funcs :=
    minimum_liquidity ::
    get_amount_out ::
    get_amount_in ::
    quote ::
    mint_initial ::
    mint_proportional ::
    burn_share ::
    k_holds ::
    k_holds_with_fee ::
    mul_div ::
    mul_div_up ::
    model_widen ::
    model_mul_lo ::
    model_mul_hi ::
    model_mul_by_limb ::
    model_div64 ::
    model_sqrt64 ::
    wide_from_u32 ::
    wide_mul32 ::
    wide_add ::
    wide_sub ::
    wide_mul_by ::
    wide_eq ::
    wide_lt ::
    wide_le ::
    wide_shr32 ::
    wide_shl1_or ::
    wide_quotient_fits ::
    wide_div_to_u32 ::
    wide_isqrt ::
    nil;
  mod_tables :=
    nil;
  mod_mems :=
    Mm {|lim_min := 1%N; lim_max := Some(1%N)|} ::
    nil;
  mod_globals :=
    Mg MUT_var (T_num T_i32) (    BI_const_num (Vi32 65536) ::
    nil) ::
    nil;
  mod_elems :=
    nil;
  mod_datas :=
    nil;
  mod_start := None;
  mod_imports :=
    nil;
  mod_exports :=
    Me "minimum_liquidity" (MED_func 0%N) ::
    Me "get_amount_out" (MED_func 1%N) ::
    Me "get_amount_in" (MED_func 2%N) ::
    Me "quote" (MED_func 3%N) ::
    Me "mint_initial" (MED_func 4%N) ::
    Me "mint_proportional" (MED_func 5%N) ::
    Me "burn_share" (MED_func 6%N) ::
    Me "k_holds" (MED_func 7%N) ::
    Me "k_holds_with_fee" (MED_func 8%N) ::
    Me "mul_div" (MED_func 9%N) ::
    Me "mul_div_up" (MED_func 10%N) ::
    Me "memory" (MED_mem 0%N) ::
    Me "__stack_pointer" (MED_global 0%N) ::
    nil;
|}.

Definition main__PairMath_hspec1 : hassert :=
  Himpl (HA_and (HA_has_type (T_local 0%N) T_i32) (HA_and (HA_has_type (T_local 1%N) T_i32) (HA_and (HA_has_type (T_local 2%N) T_i32) (HA_and (HA_has_type (T_local 3%N) T_i32) (HA_and (HA_and (HA_and (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 0%N) (T_const (Vi32 0))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 1%N) (T_const (Vi32 0))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 2%N) (T_const (Vi32 0))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_lt SX_U)) (T_local 3%N) (T_const (Vi32 10000))) (T_const (Vi32 0))))))))) (HA_and (HA_app_ok 1 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: nil)) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_lt SX_U)) (T_app 1 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: nil)) (T_local 2%N)) (T_const (Vi32 0))))).
Definition main__PairMath_hspec2 : hassert :=
  Himpl (HA_and (HA_has_type (T_local 0%N) T_i32) (HA_and (HA_has_type (T_local 1%N) T_i32) (HA_and (HA_has_type (T_local 2%N) T_i32) (HA_and (HA_has_type (T_local 3%N) T_i32) (HA_and (HA_and (HA_and (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 0%N) (T_const (Vi32 0))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 1%N) (T_const (Vi32 0))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 2%N) (T_const (Vi32 0))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_lt SX_U)) (T_local 3%N) (T_const (Vi32 10000))) (T_const (Vi32 0))))))))) (HA_and (HA_not (term_eq (T_app 8 ((T_local 0%N) :: (T_app 1 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: nil)) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: nil)) (T_const (Vi32 0)))) (Himpl (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_lt SX_U)) (T_binop T_i32 (Binop_i BOI_add) (T_app 1 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: nil)) (T_const (Vi32 1))) (T_local 2%N)) (T_const (Vi32 0)))) (term_eq (T_app 8 ((T_local 0%N) :: (T_binop T_i32 (Binop_i BOI_add) (T_app 1 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: nil)) (T_const (Vi32 1))) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: nil)) (T_const (Vi32 0))))).
Definition main__PairMath_hspec3 : hassert :=
  Himpl (HA_and (HA_has_type (T_local 0%N) T_i32) (HA_and (HA_has_type (T_local 1%N) T_i32) (HA_and (HA_has_type (T_local 2%N) T_i32) (HA_and (HA_has_type (T_local 3%N) T_i32) (HA_and (HA_and (HA_and (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 0%N) (T_const (Vi32 0))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 1%N) (T_const (Vi32 0))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 2%N) (T_const (Vi32 0))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_lt SX_U)) (T_local 3%N) (T_const (Vi32 10000))) (T_const (Vi32 0))))))))) (HA_not (term_eq (T_relop T_i64 (Relop_i ROI_eq) (T_app 11 ((T_app 1 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: nil)) :: nil)) (T_binop T_i64 (Binop_i BOI_add) (T_binop T_i64 (Binop_i BOI_mul) (T_binop T_i64 (Binop_i (BOI_div SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_const (Vi32 10000)) (T_local 3%N)) :: nil))) (T_binop T_i64 (Binop_i (BOI_shr SX_U)) (T_app 11 ((T_local 2%N) :: nil)) (T_const (Vi64 16)))) (T_binop T_i64 (Binop_i BOI_add) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 1%N) :: nil)) (T_const (Vi64 10000))) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_const (Vi32 10000)) (T_local 3%N)) :: nil))))) (T_const (Vi64 65536))) (T_binop T_i64 (Binop_i (BOI_div SX_U)) (T_binop T_i64 (Binop_i BOI_add) (T_binop T_i64 (Binop_i BOI_mul) (T_binop T_i64 (Binop_i (BOI_rem SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_const (Vi32 10000)) (T_local 3%N)) :: nil))) (T_binop T_i64 (Binop_i (BOI_shr SX_U)) (T_app 11 ((T_local 2%N) :: nil)) (T_const (Vi64 16)))) (T_binop T_i64 (Binop_i BOI_add) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 1%N) :: nil)) (T_const (Vi64 10000))) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_const (Vi32 10000)) (T_local 3%N)) :: nil))))) (T_const (Vi64 65536))) (T_binop T_i64 (Binop_i BOI_mul) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_const (Vi32 10000)) (T_local 3%N)) :: nil))) (T_binop T_i64 (Binop_i BOI_and) (T_app 11 ((T_local 2%N) :: nil)) (T_const (Vi64 65535))))) (T_binop T_i64 (Binop_i BOI_add) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 1%N) :: nil)) (T_const (Vi64 10000))) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_const (Vi32 10000)) (T_local 3%N)) :: nil))))))) (T_const (Vi32 0)))).
Definition main__PairMath_hspec4 : hassert :=
  Himpl (HA_and (HA_has_type (T_local 0%N) T_i32) (HA_and (HA_has_type (T_local 1%N) T_i32) (HA_and (HA_has_type (T_local 2%N) T_i32) (HA_and (HA_has_type (T_local 3%N) T_i32) (HA_and (HA_and (HA_and (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 0%N) (T_const (Vi32 0))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 1%N) (T_const (Vi32 0))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 2%N) (T_const (Vi32 0))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_lt SX_U)) (T_local 3%N) (T_const (Vi32 10000))) (T_const (Vi32 0))))))))) (Himpl (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_le SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_const (Vi32 10000)) (T_local 3%N)) :: nil))) (T_binop T_i64 (Binop_i (BOI_div SX_U)) (T_const (Vi64 (-1))) (T_app 11 ((T_local 2%N) :: nil)))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i64 (Relop_i ROI_eq) (T_app 11 ((T_app 1 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: nil)) :: nil)) (T_binop T_i64 (Binop_i (BOI_div SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_const (Vi32 10000)) (T_local 3%N)) :: nil))) (T_app 11 ((T_local 2%N) :: nil))) (T_binop T_i64 (Binop_i BOI_add) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 1%N) :: nil)) (T_const (Vi64 10000))) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_const (Vi32 10000)) (T_local 3%N)) :: nil)))))) (T_const (Vi32 0))))).
Definition main__PairMath_hspec5 : hassert :=
  Himpl (HA_and (HA_has_type (T_local 0%N) T_i32) (HA_and (HA_has_type (T_local 1%N) T_i32) (HA_and (HA_has_type (T_local 2%N) T_i32) (HA_and (HA_has_type (T_local 3%N) T_i32) (HA_and (HA_has_type (T_local 4%N) T_i32) (HA_and (HA_and (HA_and (HA_and (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 0%N) (T_const (Vi32 0))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_le SX_U)) (T_local 0%N) (T_local 1%N)) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 2%N) (T_const (Vi32 0))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 3%N) (T_const (Vi32 0))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_lt SX_U)) (T_local 4%N) (T_const (Vi32 10000))) (T_const (Vi32 0)))))))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_le SX_U)) (T_app 1 ((T_local 0%N) :: (T_local 2%N) :: (T_local 3%N) :: (T_local 4%N) :: nil)) (T_app 1 ((T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: (T_local 4%N) :: nil))) (T_const (Vi32 0)))).
Definition main__PairMath_hspec6 : hassert :=
  Himpl (HA_and (HA_has_type (T_local 0%N) T_i32) (HA_and (HA_has_type (T_local 1%N) T_i32) (HA_and (HA_has_type (T_local 2%N) T_i32) (HA_and (HA_has_type (T_local 3%N) T_i32) (HA_and (HA_has_type (T_local 4%N) T_i32) (HA_and (HA_and (HA_and (HA_and (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 0%N) (T_const (Vi32 0))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 1%N) (T_const (Vi32 0))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 2%N) (T_const (Vi32 0))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_le SX_U)) (T_local 3%N) (T_local 4%N)) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_lt SX_U)) (T_local 4%N) (T_const (Vi32 10000))) (T_const (Vi32 0)))))))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_le SX_U)) (T_app 1 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 4%N) :: nil)) (T_app 1 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: nil))) (T_const (Vi32 0)))).
Definition main__PairMath_hspec7 : hassert :=
  Himpl (HA_and (HA_has_type (T_local 0%N) T_i32) (HA_and (HA_has_type (T_local 1%N) T_i32) (HA_and (HA_has_type (T_local 2%N) T_i32) (HA_and (HA_has_type (T_local 3%N) T_i32) (HA_and (HA_and (HA_and (HA_and (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 0%N) (T_const (Vi32 0))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 1%N) (T_const (Vi32 0))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 2%N) (T_const (Vi32 0))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_lt SX_U)) (T_local 0%N) (T_local 2%N)) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_lt SX_U)) (T_local 3%N) (T_const (Vi32 10000))) (T_const (Vi32 0))))))))) (Himpl (HA_and (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_le SX_U)) (T_binop T_i64 (Binop_i (BOI_div SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 1%N) :: nil)) (T_app 11 ((T_local 0%N) :: nil))) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_local 2%N) (T_local 0%N)) :: nil)) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_const (Vi32 10000)) (T_local 3%N)) :: nil)))) (T_const (Vi64 429496))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_le SX_U)) (T_binop T_i64 (Binop_i BOI_add) (T_binop T_i64 (Binop_i BOI_mul) (T_binop T_i64 (Binop_i (BOI_div SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 1%N) :: nil)) (T_app 11 ((T_local 0%N) :: nil))) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_local 2%N) (T_local 0%N)) :: nil)) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_const (Vi32 10000)) (T_local 3%N)) :: nil)))) (T_const (Vi64 10000))) (T_binop T_i64 (Binop_i (BOI_div SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_binop T_i64 (Binop_i (BOI_rem SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 1%N) :: nil)) (T_app 11 ((T_local 0%N) :: nil))) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_local 2%N) (T_local 0%N)) :: nil)) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_const (Vi32 10000)) (T_local 3%N)) :: nil)))) (T_const (Vi64 10000))) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_local 2%N) (T_local 0%N)) :: nil)) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_const (Vi32 10000)) (T_local 3%N)) :: nil))))) (T_const (Vi64 4294967294))) (T_const (Vi32 0))))) (HA_and (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_ge SX_U)) (T_app 1 ((T_app 2 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: nil)) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: nil)) (T_local 0%N)) (T_const (Vi32 0)))) (HA_not (term_eq (T_app 8 ((T_app 2 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: nil)) :: (T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: nil)) (T_const (Vi32 0)))))).
Definition main__PairMath_hspec8 : hassert :=
  Himpl (HA_and (HA_has_type (T_local 0%N) T_i32) (HA_and (HA_has_type (T_local 1%N) T_i32) (HA_and (HA_has_type (T_local 2%N) T_i32) (HA_and (HA_has_type (T_local 3%N) T_i32) (HA_and (HA_and (HA_and (HA_and (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 0%N) (T_const (Vi32 0))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 1%N) (T_const (Vi32 0))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 2%N) (T_const (Vi32 0))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_lt SX_U)) (T_local 0%N) (T_local 2%N)) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_lt SX_U)) (T_local 3%N) (T_const (Vi32 10000))) (T_const (Vi32 0))))))))) (Himpl (HA_and (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_le SX_U)) (T_binop T_i64 (Binop_i (BOI_div SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 1%N) :: nil)) (T_app 11 ((T_local 0%N) :: nil))) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_local 2%N) (T_local 0%N)) :: nil)) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_const (Vi32 10000)) (T_local 3%N)) :: nil)))) (T_const (Vi64 429496))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_le SX_U)) (T_binop T_i64 (Binop_i BOI_add) (T_binop T_i64 (Binop_i BOI_mul) (T_binop T_i64 (Binop_i (BOI_div SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 1%N) :: nil)) (T_app 11 ((T_local 0%N) :: nil))) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_local 2%N) (T_local 0%N)) :: nil)) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_const (Vi32 10000)) (T_local 3%N)) :: nil)))) (T_const (Vi64 10000))) (T_binop T_i64 (Binop_i (BOI_div SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_binop T_i64 (Binop_i (BOI_rem SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 1%N) :: nil)) (T_app 11 ((T_local 0%N) :: nil))) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_local 2%N) (T_local 0%N)) :: nil)) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_const (Vi32 10000)) (T_local 3%N)) :: nil)))) (T_const (Vi64 10000))) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_local 2%N) (T_local 0%N)) :: nil)) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_const (Vi32 10000)) (T_local 3%N)) :: nil))))) (T_const (Vi64 4294967294))) (T_const (Vi32 0))))) (Himpl (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_app 2 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: nil)) (T_const (Vi32 2))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_lt SX_U)) (T_app 1 ((T_binop T_i32 (Binop_i BOI_sub) (T_app 2 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: nil)) (T_const (Vi32 2))) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: nil)) (T_local 0%N)) (T_const (Vi32 0)))))).
Definition main__PairMath_hspec9 : hassert :=
  Himpl (HA_and (HA_has_type (T_local 0%N) T_i32) (HA_and (HA_has_type (T_local 1%N) T_i32) (HA_and (HA_has_type (T_local 2%N) T_i32) (HA_and (HA_has_type (T_local 3%N) T_i32) (HA_and (HA_and (HA_and (HA_and (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 0%N) (T_const (Vi32 0))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 1%N) (T_const (Vi32 0))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 2%N) (T_const (Vi32 0))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_lt SX_U)) (T_local 0%N) (T_local 2%N)) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_lt SX_U)) (T_local 3%N) (T_const (Vi32 10000))) (T_const (Vi32 0))))))))) (Himpl (HA_and (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_le SX_U)) (T_binop T_i64 (Binop_i (BOI_div SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 1%N) :: nil)) (T_app 11 ((T_local 0%N) :: nil))) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_local 2%N) (T_local 0%N)) :: nil)) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_const (Vi32 10000)) (T_local 3%N)) :: nil)))) (T_const (Vi64 429496))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_le SX_U)) (T_binop T_i64 (Binop_i BOI_add) (T_binop T_i64 (Binop_i BOI_mul) (T_binop T_i64 (Binop_i (BOI_div SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 1%N) :: nil)) (T_app 11 ((T_local 0%N) :: nil))) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_local 2%N) (T_local 0%N)) :: nil)) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_const (Vi32 10000)) (T_local 3%N)) :: nil)))) (T_const (Vi64 10000))) (T_binop T_i64 (Binop_i (BOI_div SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_binop T_i64 (Binop_i (BOI_rem SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 1%N) :: nil)) (T_app 11 ((T_local 0%N) :: nil))) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_local 2%N) (T_local 0%N)) :: nil)) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_const (Vi32 10000)) (T_local 3%N)) :: nil)))) (T_const (Vi64 10000))) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_local 2%N) (T_local 0%N)) :: nil)) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_const (Vi32 10000)) (T_local 3%N)) :: nil))))) (T_const (Vi64 4294967294))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i64 (Relop_i ROI_eq) (T_app 11 ((T_app 2 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: nil)) :: nil)) (T_binop T_i64 (Binop_i BOI_add) (T_binop T_i64 (Binop_i BOI_add) (T_binop T_i64 (Binop_i BOI_mul) (T_binop T_i64 (Binop_i (BOI_div SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 1%N) :: nil)) (T_app 11 ((T_local 0%N) :: nil))) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_local 2%N) (T_local 0%N)) :: nil)) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_const (Vi32 10000)) (T_local 3%N)) :: nil)))) (T_const (Vi64 10000))) (T_binop T_i64 (Binop_i (BOI_div SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_binop T_i64 (Binop_i (BOI_rem SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 1%N) :: nil)) (T_app 11 ((T_local 0%N) :: nil))) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_local 2%N) (T_local 0%N)) :: nil)) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_const (Vi32 10000)) (T_local 3%N)) :: nil)))) (T_const (Vi64 10000))) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_local 2%N) (T_local 0%N)) :: nil)) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_const (Vi32 10000)) (T_local 3%N)) :: nil))))) (T_const (Vi64 1)))) (T_const (Vi32 0))))).
Definition main__PairMath_hspec10 : hassert :=
  Himpl (HA_and (HA_has_type (T_local 0%N) T_i32) (HA_and (HA_has_type (T_local 1%N) T_i32) (HA_and (HA_has_type (T_local 2%N) T_i32) (HA_and (HA_has_type (T_local 3%N) T_i32) (HA_and (HA_and (HA_and (HA_and (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 0%N) (T_const (Vi32 0))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 1%N) (T_const (Vi32 0))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 2%N) (T_const (Vi32 0))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_lt SX_U)) (T_local 0%N) (T_local 2%N)) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_lt SX_U)) (T_local 3%N) (T_const (Vi32 10000))) (T_const (Vi32 0))))))))) (Himpl (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_le SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 1%N) :: nil)) (T_app 11 ((T_local 0%N) :: nil))) (T_const (Vi64 1844674407370955))) (T_const (Vi32 0)))) (Himpl (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_le SX_U)) (T_binop T_i64 (Binop_i (BOI_div SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 1%N) :: nil)) (T_app 11 ((T_local 0%N) :: nil))) (T_const (Vi64 10000))) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_local 2%N) (T_local 0%N)) :: nil)) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_const (Vi32 10000)) (T_local 3%N)) :: nil)))) (T_const (Vi64 4294967294))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i64 (Relop_i ROI_eq) (T_app 11 ((T_app 2 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: nil)) :: nil)) (T_binop T_i64 (Binop_i BOI_add) (T_binop T_i64 (Binop_i (BOI_div SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 1%N) :: nil)) (T_app 11 ((T_local 0%N) :: nil))) (T_const (Vi64 10000))) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_local 2%N) (T_local 0%N)) :: nil)) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_const (Vi32 10000)) (T_local 3%N)) :: nil)))) (T_const (Vi64 1)))) (T_const (Vi32 0)))))).
Definition main__PairMath_hspec11 : hassert :=
  Himpl (HA_and (HA_has_type (T_local 0%N) T_i32) (HA_and (HA_has_type (T_local 1%N) T_i32) (HA_and (HA_has_type (T_local 2%N) T_i32) (HA_and (HA_and (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 0%N) (T_const (Vi32 0))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 1%N) (T_const (Vi32 0))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 2%N) (T_const (Vi32 0))) (T_const (Vi32 0)))))))) (Himpl (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_lt SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_local 2%N) :: nil))) (T_binop T_i64 (Binop_i BOI_shl) (T_app 11 ((T_local 1%N) :: nil)) (T_const (Vi64 32)))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i64 (Relop_i ROI_eq) (T_app 11 ((T_app 3 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: nil)) :: nil)) (T_binop T_i64 (Binop_i (BOI_div SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_local 2%N) :: nil))) (T_app 11 ((T_local 1%N) :: nil)))) (T_const (Vi32 0))))).
Definition main__PairMath_hspec12 : hassert :=
  Himpl (HA_and (HA_has_type (T_local 0%N) T_i32) (HA_and (HA_has_type (T_local 1%N) T_i32) (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_ge SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_local 1%N) :: nil))) (T_const (Vi64 1002001))) (T_const (Vi32 0)))))) (HA_and (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_le SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_binop T_i64 (Binop_i BOI_add) (T_app 11 ((T_app 4 ((T_local 0%N) :: (T_local 1%N) :: nil)) :: nil)) (T_app 11 ((T_app 0 nil) :: nil))) (T_binop T_i64 (Binop_i BOI_add) (T_app 11 ((T_app 4 ((T_local 0%N) :: (T_local 1%N) :: nil)) :: nil)) (T_app 11 ((T_app 0 nil) :: nil)))) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_local 1%N) :: nil)))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_le SX_U)) (T_binop T_i64 (Binop_i BOI_sub) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_local 1%N) :: nil))) (T_binop T_i64 (Binop_i BOI_mul) (T_binop T_i64 (Binop_i BOI_add) (T_app 11 ((T_app 4 ((T_local 0%N) :: (T_local 1%N) :: nil)) :: nil)) (T_app 11 ((T_app 0 nil) :: nil))) (T_binop T_i64 (Binop_i BOI_add) (T_app 11 ((T_app 4 ((T_local 0%N) :: (T_local 1%N) :: nil)) :: nil)) (T_app 11 ((T_app 0 nil) :: nil))))) (T_binop T_i64 (Binop_i BOI_mul) (T_const (Vi64 2)) (T_binop T_i64 (Binop_i BOI_add) (T_app 11 ((T_app 4 ((T_local 0%N) :: (T_local 1%N) :: nil)) :: nil)) (T_app 11 ((T_app 0 nil) :: nil))))) (T_const (Vi32 0))))).
Definition main__PairMath_hspec13 : hassert :=
  Himpl (HA_and (HA_has_type (T_local 0%N) T_i32) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 0%N) (T_const (Vi32 1000))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i ROI_eq) (T_app 4 ((T_local 0%N) :: (T_local 0%N) :: nil)) (T_binop T_i32 (Binop_i BOI_sub) (T_local 0%N) (T_app 0 nil))) (T_const (Vi32 0)))).
Definition main__PairMath_hspec14 : hassert :=
  Himpl (HA_and (HA_has_type (T_local 0%N) T_i32) (HA_and (HA_has_type (T_local 1%N) T_i32) (HA_and (HA_has_type (T_local 2%N) T_i32) (HA_and (HA_has_type (T_local 3%N) T_i32) (HA_and (HA_has_type (T_local 4%N) T_i32) (HA_and (HA_and (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 2%N) (T_const (Vi32 0))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 3%N) (T_const (Vi32 0))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 4%N) (T_const (Vi32 0))) (T_const (Vi32 0)))))))))) (Himpl (HA_and (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_lt SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_local 4%N) :: nil))) (T_binop T_i64 (Binop_i BOI_shl) (T_app 11 ((T_local 2%N) :: nil)) (T_const (Vi64 32)))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_lt SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 1%N) :: nil)) (T_app 11 ((T_local 4%N) :: nil))) (T_binop T_i64 (Binop_i BOI_shl) (T_app 11 ((T_local 3%N) :: nil)) (T_const (Vi64 32)))) (T_const (Vi32 0))))) (Himpl (HA_and (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_ge SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_local 4%N) :: nil))) (T_app 11 ((T_local 2%N) :: nil))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_ge SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 1%N) :: nil)) (T_app 11 ((T_local 4%N) :: nil))) (T_app 11 ((T_local 3%N) :: nil))) (T_const (Vi32 0))))) (HA_and (HA_and (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_le SX_U)) (T_app 11 ((T_app 5 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: (T_local 4%N) :: nil)) :: nil)) (T_binop T_i64 (Binop_i (BOI_div SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_local 4%N) :: nil))) (T_app 11 ((T_local 2%N) :: nil)))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_le SX_U)) (T_app 11 ((T_app 5 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: (T_local 4%N) :: nil)) :: nil)) (T_binop T_i64 (Binop_i (BOI_div SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 1%N) :: nil)) (T_app 11 ((T_local 4%N) :: nil))) (T_app 11 ((T_local 3%N) :: nil)))) (T_const (Vi32 0))))) (Hor (HA_not (term_eq (T_relop T_i64 (Relop_i ROI_eq) (T_app 11 ((T_app 5 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: (T_local 4%N) :: nil)) :: nil)) (T_binop T_i64 (Binop_i (BOI_div SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_local 4%N) :: nil))) (T_app 11 ((T_local 2%N) :: nil)))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i64 (Relop_i ROI_eq) (T_app 11 ((T_app 5 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: (T_local 4%N) :: nil)) :: nil)) (T_binop T_i64 (Binop_i (BOI_div SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 1%N) :: nil)) (T_app 11 ((T_local 4%N) :: nil))) (T_app 11 ((T_local 3%N) :: nil)))) (T_const (Vi32 0)))))))).
Definition main__PairMath_hspec15 : hassert :=
  Himpl (HA_and (HA_has_type (T_local 0%N) T_i32) (HA_and (HA_has_type (T_local 1%N) T_i32) (HA_and (HA_has_type (T_local 2%N) T_i32) (HA_and (HA_has_type (T_local 3%N) T_i32) (HA_and (HA_has_type (T_local 4%N) T_i32) (HA_and (HA_and (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 2%N) (T_const (Vi32 0))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 3%N) (T_const (Vi32 0))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 4%N) (T_const (Vi32 0))) (T_const (Vi32 0)))))))))) (Himpl (HA_not (term_eq (T_relop T_i32 (Relop_i ROI_ne) (T_relop T_i64 (Relop_i (ROI_lt SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_local 4%N) :: nil))) (T_binop T_i64 (Binop_i BOI_shl) (T_app 11 ((T_local 2%N) :: nil)) (T_const (Vi64 32)))) (T_relop T_i64 (Relop_i (ROI_lt SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 1%N) :: nil)) (T_app 11 ((T_local 4%N) :: nil))) (T_binop T_i64 (Binop_i BOI_shl) (T_app 11 ((T_local 3%N) :: nil)) (T_const (Vi64 32))))) (T_const (Vi32 0)))) (Himpl (HA_and (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_ge SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_local 4%N) :: nil))) (T_app 11 ((T_local 2%N) :: nil))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_ge SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 1%N) :: nil)) (T_app 11 ((T_local 4%N) :: nil))) (T_app 11 ((T_local 3%N) :: nil))) (T_const (Vi32 0))))) (HA_and (Himpl (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_lt SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_local 4%N) :: nil))) (T_binop T_i64 (Binop_i BOI_shl) (T_app 11 ((T_local 2%N) :: nil)) (T_const (Vi64 32)))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i64 (Relop_i ROI_eq) (T_app 11 ((T_app 5 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: (T_local 4%N) :: nil)) :: nil)) (T_binop T_i64 (Binop_i (BOI_div SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_local 4%N) :: nil))) (T_app 11 ((T_local 2%N) :: nil)))) (T_const (Vi32 0))))) (Himpl (term_eq (T_relop T_i64 (Relop_i (ROI_lt SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_local 4%N) :: nil))) (T_binop T_i64 (Binop_i BOI_shl) (T_app 11 ((T_local 2%N) :: nil)) (T_const (Vi64 32)))) (T_const (Vi32 0))) (HA_not (term_eq (T_relop T_i64 (Relop_i ROI_eq) (T_app 11 ((T_app 5 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: (T_local 4%N) :: nil)) :: nil)) (T_binop T_i64 (Binop_i (BOI_div SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 1%N) :: nil)) (T_app 11 ((T_local 4%N) :: nil))) (T_app 11 ((T_local 3%N) :: nil)))) (T_const (Vi32 0)))))))).
Definition main__PairMath_hspec16 : hassert :=
  Himpl (HA_and (HA_has_type (T_local 0%N) T_i32) (HA_and (HA_has_type (T_local 1%N) T_i32) (HA_and (HA_has_type (T_local 2%N) T_i32) (HA_and (HA_and (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 2%N) (T_const (Vi32 0))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_le SX_U)) (T_local 0%N) (T_local 2%N)) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_ge SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_local 1%N) :: nil))) (T_app 11 ((T_local 2%N) :: nil))) (T_const (Vi32 0)))))))) (HA_and (HA_not (term_eq (T_relop T_i64 (Relop_i ROI_eq) (T_app 11 ((T_app 6 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: nil)) :: nil)) (T_binop T_i64 (Binop_i (BOI_div SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_local 1%N) :: nil))) (T_app 11 ((T_local 2%N) :: nil)))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_le SX_U)) (T_app 6 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: nil)) (T_local 1%N)) (T_const (Vi32 0))))).
Definition main__PairMath_hspec17 : hassert :=
  Himpl (HA_and (HA_has_type (T_local 0%N) T_i32) (HA_and (HA_has_type (T_local 1%N) T_i32) (HA_and (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 1%N) (T_const (Vi32 0))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 0%N) (T_const (Vi32 0))) (T_const (Vi32 0))))))) (HA_not (term_eq (T_relop T_i32 (Relop_i ROI_eq) (T_app 6 ((T_local 1%N) :: (T_local 0%N) :: (T_local 1%N) :: nil)) (T_local 0%N)) (T_const (Vi32 0)))).
Definition main__PairMath_hspec18 : hassert :=
  Himpl (HA_and (HA_has_type (T_local 0%N) T_i32) (HA_and (HA_has_type (T_local 1%N) T_i32) (HA_and (HA_has_type (T_local 2%N) T_i32) (HA_and (HA_has_type (T_local 3%N) T_i32) (HA_and (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 2%N) (T_const (Vi32 0))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_lt SX_U)) (T_local 1%N) (T_local 3%N)) (T_const (Vi32 0))))))))) (HA_not (term_eq (T_relop T_i32 (Relop_i ROI_eq) (T_app 7 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: nil)) (T_relop T_i64 (Relop_i (ROI_ge SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_local 3%N) (T_local 1%N)) :: nil))) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 2%N) :: nil)) (T_app 11 ((T_local 1%N) :: nil))))) (T_const (Vi32 0)))).
Definition main__PairMath_hspec19 : hassert :=
  Himpl (HA_and (HA_has_type (T_local 0%N) T_i32) (HA_and (HA_has_type (T_local 1%N) T_i32) (HA_and (HA_has_type (T_local 2%N) T_i32) (HA_and (HA_has_type (T_local 3%N) T_i32) (HA_and (HA_has_type (T_local 4%N) T_i32) (HA_and (HA_and (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_lt SX_U)) (T_local 4%N) (T_const (Vi32 10000))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 2%N) (T_const (Vi32 0))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_lt SX_U)) (T_local 1%N) (T_local 3%N)) (T_const (Vi32 0)))))))))) (HA_not (term_eq (T_relop T_i32 (Relop_i ROI_eq) (T_app 8 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: (T_local 4%N) :: nil)) (T_relop T_i64 (Relop_i (ROI_ge SX_U)) (T_binop T_i64 (Binop_i BOI_add) (T_binop T_i64 (Binop_i BOI_mul) (T_binop T_i64 (Binop_i (BOI_div SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_local 3%N) (T_local 1%N)) :: nil))) (T_const (Vi64 10000))) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_const (Vi32 10000)) (T_local 4%N)) :: nil))) (T_binop T_i64 (Binop_i (BOI_div SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_binop T_i64 (Binop_i (BOI_rem SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_local 3%N) (T_local 1%N)) :: nil))) (T_const (Vi64 10000))) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_const (Vi32 10000)) (T_local 4%N)) :: nil))) (T_const (Vi64 10000)))) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 2%N) :: nil)) (T_app 11 ((T_local 1%N) :: nil))))) (T_const (Vi32 0)))).
Definition main__PairMath_hspec20 : hassert :=
  Himpl (HA_and (HA_has_type (T_local 0%N) T_i32) (HA_and (HA_has_type (T_local 1%N) T_i32) (HA_and (HA_has_type (T_local 2%N) T_i32) (HA_and (HA_has_type (T_local 3%N) T_i32) (HA_and (HA_has_type (T_local 4%N) T_i32) (HA_and (HA_and (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_lt SX_U)) (T_local 4%N) (T_const (Vi32 10000))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 2%N) (T_const (Vi32 0))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_lt SX_U)) (T_local 1%N) (T_local 3%N)) (T_const (Vi32 0)))))))))) (Himpl (HA_and (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_le SX_U)) (T_binop T_i64 (Binop_i BOI_sub) (T_binop T_i64 (Binop_i BOI_mul) (T_binop T_i64 (Binop_i BOI_add) (T_app 11 ((T_local 2%N) :: nil)) (T_app 11 ((T_local 0%N) :: nil))) (T_const (Vi64 10000))) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_local 4%N) :: nil)))) (T_binop T_i64 (Binop_i (BOI_div SX_U)) (T_const (Vi64 (-1))) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_local 3%N) (T_local 1%N)) :: nil)))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_le SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 2%N) :: nil)) (T_app 11 ((T_local 3%N) :: nil))) (T_const (Vi64 1844674407370955))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i ROI_eq) (T_app 8 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: (T_local 4%N) :: nil)) (T_relop T_i64 (Relop_i (ROI_ge SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_binop T_i64 (Binop_i BOI_sub) (T_binop T_i64 (Binop_i BOI_mul) (T_binop T_i64 (Binop_i BOI_add) (T_app 11 ((T_local 2%N) :: nil)) (T_app 11 ((T_local 0%N) :: nil))) (T_const (Vi64 10000))) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_local 4%N) :: nil)))) (T_app 11 ((T_binop T_i32 (Binop_i BOI_sub) (T_local 3%N) (T_local 1%N)) :: nil))) (T_binop T_i64 (Binop_i BOI_mul) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 2%N) :: nil)) (T_app 11 ((T_local 3%N) :: nil))) (T_const (Vi64 10000))))) (T_const (Vi32 0))))).
Definition main__PairMath_hspec21 : hassert :=
  Himpl (HA_and (HA_has_type (T_local 0%N) T_i32) (HA_and (HA_has_type (T_local 1%N) T_i32) (HA_and (HA_has_type (T_local 2%N) T_i32) (HA_and (HA_has_type (T_local 3%N) T_i32) (HA_and (HA_and (HA_and (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 0%N) (T_const (Vi32 0))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 1%N) (T_const (Vi32 0))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 2%N) (T_const (Vi32 0))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_lt SX_U)) (T_local 3%N) (T_const (Vi32 10000))) (T_const (Vi32 0))))))))) (HA_and (HA_not (term_eq (T_app 7 ((T_local 0%N) :: (T_app 1 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: nil)) :: (T_local 1%N) :: (T_local 2%N) :: nil)) (T_const (Vi32 0)))) (HA_not (term_eq (T_app 8 ((T_local 0%N) :: (T_app 1 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: nil)) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: nil)) (T_const (Vi32 0))))).
Definition main__PairMath_hspec22 : hassert :=
  Himpl (HA_and (HA_has_type (T_local 0%N) T_i32) (HA_and (HA_has_type (T_local 1%N) T_i32) (HA_and (HA_has_type (T_local 2%N) T_i32) (HA_and (HA_has_type (T_local 3%N) T_i32) (HA_and (HA_and (HA_and (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 1%N) (T_const (Vi32 0))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 0%N) (T_const (Vi32 0))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_lt SX_U)) (T_local 0%N) (T_local 2%N)) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_lt SX_U)) (T_local 3%N) (T_const (Vi32 10000))) (T_const (Vi32 0))))))))) (HA_and (term_eq (T_app 7 ((T_const (Vi32 0)) :: (T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: nil)) (T_const (Vi32 0))) (term_eq (T_app 8 ((T_const (Vi32 0)) :: (T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_local 3%N) :: nil)) (T_const (Vi32 0)))).
Definition main__PairMath_hspec23 : hassert :=
  Himpl (HA_and (HA_has_type (T_local 0%N) T_i32) (HA_and (HA_has_type (T_local 1%N) T_i32) (HA_and (HA_has_type (T_local 2%N) T_i32) (HA_and (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 2%N) (T_const (Vi32 0))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_lt SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_local 1%N) :: nil))) (T_binop T_i64 (Binop_i BOI_shl) (T_app 11 ((T_local 2%N) :: nil)) (T_const (Vi64 32)))) (T_const (Vi32 0)))))))) (HA_not (term_eq (T_relop T_i64 (Relop_i ROI_eq) (T_app 11 ((T_app 9 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: nil)) :: nil)) (T_binop T_i64 (Binop_i (BOI_div SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_local 1%N) :: nil))) (T_app 11 ((T_local 2%N) :: nil)))) (T_const (Vi32 0)))).
Definition main__PairMath_hspec24 : hassert :=
  Himpl (HA_and (HA_has_type (T_local 0%N) T_i32) (HA_and (HA_has_type (T_local 1%N) T_i32) (HA_and (HA_has_type (T_local 2%N) T_i32) (HA_and (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_gt SX_U)) (T_local 2%N) (T_const (Vi32 0))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_lt SX_U)) (T_binop T_i64 (Binop_i BOI_sub) (T_binop T_i64 (Binop_i BOI_add) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_local 1%N) :: nil))) (T_app 11 ((T_local 2%N) :: nil))) (T_const (Vi64 1))) (T_binop T_i64 (Binop_i BOI_shl) (T_app 11 ((T_local 2%N) :: nil)) (T_const (Vi64 32)))) (T_const (Vi32 0)))))))) (HA_not (term_eq (T_relop T_i64 (Relop_i ROI_eq) (T_app 11 ((T_app 10 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: nil)) :: nil)) (T_binop T_i64 (Binop_i (BOI_div SX_U)) (T_binop T_i64 (Binop_i BOI_sub) (T_binop T_i64 (Binop_i BOI_add) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_local 1%N) :: nil))) (T_app 11 ((T_local 2%N) :: nil))) (T_const (Vi64 1))) (T_app 11 ((T_local 2%N) :: nil)))) (T_const (Vi32 0)))).
Definition main__PairMath_hspec25 : hassert :=
  HA_and (HA_not (term_eq (T_relop T_i32 (Relop_i ROI_eq) (T_app 0 nil) (T_const (Vi32 1000))) (T_const (Vi32 0)))) (HA_and (HA_not (term_eq (T_relop T_i32 (Relop_i ROI_eq) (T_app 1 ((T_const (Vi32 1000)) :: (T_const (Vi32 100000)) :: (T_const (Vi32 100000)) :: (T_const (Vi32 30)) :: nil)) (T_const (Vi32 987))) (T_const (Vi32 0)))) (HA_and (HA_not (term_eq (T_relop T_i32 (Relop_i ROI_eq) (T_app 2 ((T_const (Vi32 987)) :: (T_const (Vi32 100000)) :: (T_const (Vi32 100000)) :: (T_const (Vi32 30)) :: nil)) (T_const (Vi32 1000))) (T_const (Vi32 0)))) (HA_and (HA_not (term_eq (T_relop T_i32 (Relop_i ROI_eq) (T_app 1 ((T_const (Vi32 1)) :: (T_const (Vi32 1000000)) :: (T_const (Vi32 1000)) :: (T_const (Vi32 30)) :: nil)) (T_const (Vi32 0))) (T_const (Vi32 0)))) (HA_and (HA_not (term_eq (T_relop T_i32 (Relop_i ROI_eq) (T_app 4 ((T_const (Vi32 1000000)) :: (T_const (Vi32 4000000)) :: nil)) (T_const (Vi32 1999000))) (T_const (Vi32 0)))) (HA_and (HA_not (term_eq (T_relop T_i32 (Relop_i ROI_eq) (T_app 3 ((T_const (Vi32 100)) :: (T_const (Vi32 1000)) :: (T_const (Vi32 3000)) :: nil)) (T_const (Vi32 300))) (T_const (Vi32 0)))) (HA_and (HA_not (term_eq (T_relop T_i32 (Relop_i ROI_eq) (T_app 9 ((T_const (Vi32 (-1))) :: (T_const (Vi32 (-1))) :: (T_const (Vi32 (-1))) :: nil)) (T_const (Vi32 (-1)))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i32 (Relop_i ROI_eq) (T_app 10 ((T_const (Vi32 (-1))) :: (T_const (Vi32 (-1))) :: (T_const (Vi32 (-1))) :: nil)) (T_const (Vi32 (-1)))) (T_const (Vi32 0)))))))))).
Definition main__PairMath_specs : list hassert := (main__PairMath_hspec1 :: main__PairMath_hspec2 :: main__PairMath_hspec3 :: main__PairMath_hspec4 :: main__PairMath_hspec5 :: main__PairMath_hspec6 :: main__PairMath_hspec7 :: main__PairMath_hspec8 :: main__PairMath_hspec9 :: main__PairMath_hspec10 :: main__PairMath_hspec11 :: main__PairMath_hspec12 :: main__PairMath_hspec13 :: main__PairMath_hspec14 :: main__PairMath_hspec15 :: main__PairMath_hspec16 :: main__PairMath_hspec17 :: main__PairMath_hspec18 :: main__PairMath_hspec19 :: main__PairMath_hspec20 :: main__PairMath_hspec21 :: main__PairMath_hspec22 :: main__PairMath_hspec23 :: main__PairMath_hspec24 :: main__PairMath_hspec25 :: nil).

Definition main__model_WideArith_hspec1 : hassert :=
  HA_not (term_eq (T_relop T_i64 (Relop_i ROI_eq) (T_app 11 ((T_const (Vi32 0)) :: nil)) (T_const (Vi64 0))) (T_const (Vi32 0))).
Definition main__model_WideArith_hspec2 : hassert :=
  Himpl (HA_and (HA_has_type (T_local 0%N) T_i32) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_lt SX_U)) (T_local 0%N) (T_const (Vi32 (-1)))) (T_const (Vi32 0))))) (HA_not (term_eq (T_relop T_i64 (Relop_i ROI_eq) (T_app 11 ((T_binop T_i32 (Binop_i BOI_add) (T_local 0%N) (T_const (Vi32 1))) :: nil)) (T_binop T_i64 (Binop_i BOI_add) (T_app 11 ((T_local 0%N) :: nil)) (T_const (Vi64 1)))) (T_const (Vi32 0)))).
Definition main__model_WideArith_hspec3 : hassert :=
  Himpl (HA_has_type (T_local 0%N) T_i32) (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_le SX_U)) (T_app 11 ((T_local 0%N) :: nil)) (T_const (Vi64 4294967295))) (T_const (Vi32 0)))).
Definition main__model_WideArith_hspec4 : hassert :=
  Himpl (HA_and (HA_has_type (T_local 0%N) T_i32) (HA_has_type (T_local 1%N) T_i32)) (HA_not (term_eq (T_relop T_i64 (Relop_i ROI_eq) (T_binop T_i64 (Binop_i BOI_or) (T_binop T_i64 (Binop_i BOI_shl) (T_app 11 ((T_app 13 ((T_local 0%N) :: (T_local 1%N) :: nil)) :: nil)) (T_const (Vi64 32))) (T_app 11 ((T_app 12 ((T_local 0%N) :: (T_local 1%N) :: nil)) :: nil))) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_local 1%N) :: nil)))) (T_const (Vi32 0)))).
Definition main__model_WideArith_hspec5 : hassert :=
  Himpl (HA_and (HA_has_type (T_local 0%N) T_i32) (HA_and (HA_has_type (T_local 1%N) T_i32) (HA_has_type (T_local 2%N) T_i32))) (HA_and (HA_not (term_eq (T_relop T_i64 (Relop_i ROI_eq) (T_binop T_i64 (Binop_i BOI_or) (T_binop T_i64 (Binop_i BOI_shl) (T_app 11 ((T_app 14 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_const (Vi32 1)) :: nil)) :: nil)) (T_const (Vi64 32))) (T_app 11 ((T_app 14 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_const (Vi32 0)) :: nil)) :: nil))) (T_binop T_i64 (Binop_i BOI_mul) (T_binop T_i64 (Binop_i BOI_or) (T_binop T_i64 (Binop_i BOI_shl) (T_app 11 ((T_local 0%N) :: nil)) (T_const (Vi64 32))) (T_app 11 ((T_local 1%N) :: nil))) (T_app 11 ((T_local 2%N) :: nil)))) (T_const (Vi32 0)))) (HA_and (HA_not (term_eq (T_relop T_i64 (Relop_i ROI_eq) (T_app 11 ((T_app 14 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_const (Vi32 2)) :: nil)) :: nil)) (T_binop T_i64 (Binop_i (BOI_shr SX_U)) (T_binop T_i64 (Binop_i BOI_add) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 0%N) :: nil)) (T_app 11 ((T_local 2%N) :: nil))) (T_binop T_i64 (Binop_i (BOI_shr SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_local 1%N) :: nil)) (T_app 11 ((T_local 2%N) :: nil))) (T_const (Vi64 32)))) (T_const (Vi64 32)))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i32 (Relop_i ROI_eq) (T_app 14 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: (T_const (Vi32 3)) :: nil)) (T_const (Vi32 0))) (T_const (Vi32 0)))))).
Definition main__model_WideArith_hspec6 : hassert :=
  Himpl (HA_and (HA_has_type (T_local 0%N) T_i32) (HA_and (HA_has_type (T_local 1%N) T_i32) (HA_and (HA_has_type (T_local 2%N) T_i32) (HA_not (term_eq (T_relop T_i32 (Relop_i (ROI_lt SX_U)) (T_local 0%N) (T_local 2%N)) (T_const (Vi32 0))))))) (HA_and (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_le SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_app 15 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: nil)) :: nil)) (T_app 11 ((T_local 2%N) :: nil))) (T_binop T_i64 (Binop_i BOI_or) (T_binop T_i64 (Binop_i BOI_shl) (T_app 11 ((T_local 0%N) :: nil)) (T_const (Vi64 32))) (T_app 11 ((T_local 1%N) :: nil)))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_lt SX_U)) (T_binop T_i64 (Binop_i BOI_sub) (T_binop T_i64 (Binop_i BOI_or) (T_binop T_i64 (Binop_i BOI_shl) (T_app 11 ((T_local 0%N) :: nil)) (T_const (Vi64 32))) (T_app 11 ((T_local 1%N) :: nil))) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_app 15 ((T_local 0%N) :: (T_local 1%N) :: (T_local 2%N) :: nil)) :: nil)) (T_app 11 ((T_local 2%N) :: nil)))) (T_app 11 ((T_local 2%N) :: nil))) (T_const (Vi32 0))))).
Definition main__model_WideArith_hspec7 : hassert :=
  Himpl (HA_and (HA_has_type (T_local 0%N) T_i32) (HA_has_type (T_local 1%N) T_i32)) (HA_and (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_le SX_U)) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_app 16 ((T_local 0%N) :: (T_local 1%N) :: nil)) :: nil)) (T_app 11 ((T_app 16 ((T_local 0%N) :: (T_local 1%N) :: nil)) :: nil))) (T_binop T_i64 (Binop_i BOI_or) (T_binop T_i64 (Binop_i BOI_shl) (T_app 11 ((T_local 0%N) :: nil)) (T_const (Vi64 32))) (T_app 11 ((T_local 1%N) :: nil)))) (T_const (Vi32 0)))) (HA_not (term_eq (T_relop T_i64 (Relop_i (ROI_le SX_U)) (T_binop T_i64 (Binop_i BOI_sub) (T_binop T_i64 (Binop_i BOI_or) (T_binop T_i64 (Binop_i BOI_shl) (T_app 11 ((T_local 0%N) :: nil)) (T_const (Vi64 32))) (T_app 11 ((T_local 1%N) :: nil))) (T_binop T_i64 (Binop_i BOI_mul) (T_app 11 ((T_app 16 ((T_local 0%N) :: (T_local 1%N) :: nil)) :: nil)) (T_app 11 ((T_app 16 ((T_local 0%N) :: (T_local 1%N) :: nil)) :: nil)))) (T_binop T_i64 (Binop_i BOI_mul) (T_const (Vi64 2)) (T_app 11 ((T_app 16 ((T_local 0%N) :: (T_local 1%N) :: nil)) :: nil)))) (T_const (Vi32 0))))).
Definition main__model_WideArith_specs : list hassert := (main__model_WideArith_hspec1 :: main__model_WideArith_hspec2 :: main__model_WideArith_hspec3 :: main__model_WideArith_hspec4 :: main__model_WideArith_hspec5 :: main__model_WideArith_hspec6 :: main__model_WideArith_hspec7 :: nil).

Section Host.
Context `{ho: host}.

Theorem valid_main : ValidModule main.
Proof.
  (* TODO: fill the proof *)
Admitted.

Theorem valid_main__PairMath : ValidSpec main main__PairMath_specs.
Proof.
  (* TODO: fill the proof *)
Admitted.

Theorem valid_main__model_WideArith : ValidSpec main main__model_WideArith_specs.
Proof.
  (* TODO: fill the proof *)
Admitted.

End Host.
