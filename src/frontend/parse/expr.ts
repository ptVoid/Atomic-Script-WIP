import { ParserStmt } from "./stmt.ts";
import { Null, Num, Str, Bool, Id, Property, Object } from "../AST/values.ts";
import { Expr } from "../AST/stmts.ts";
import { BinaryExpr, AssignExpr } from "../AST/exprs.ts";
import { Type, Ion } from "../Ion.ts";

export class ParserExpr extends ParserStmt {

   protected parse_assign_expr() : Expr {
      const left = this.parse_obj_expr();

      if(this.at().type === Type.setter) {
         this.take();

         const value = this.parse_expr();
         
         return {
            type: "AssignExpr",
            assigne: left,
            value: value,
            line: this.line,
            colmun: this.colmun
         } as AssignExpr;
      }

      return left;
   }



   protected parse_obj_expr() : Expr {
      if(this.at().type != Type.OpenBrace) {
         return this.parse_mathmatic_expr();
      }

      this.take();

      let properties: Property[] = [];

      while(this.notEOF() && this.at().type != Type.CloseBrace) {
         let key = this.except(Type.id).value;


         let property: Property;
         
         if(this.at().type === Type.Comma) {
            this.take();
            property = { 
               type: "Property", 
               key: key, 
               value: null,
               line: this.line,
               colmun: this.colmun
            }
            properties.push(property);
            continue;
         }
         else if(this.at().type === Type.CloseBrace) {
            property = {
               type: "Property",
               key: key,
               value: null,
               line: this.line,
               colmun: this.colmun
            }
            properties.push(property);
            continue;
         }
         this.except(Type.Colon);

         let value = this.parse_expr();

         property = {
            type: "Property",
            key: key,
            value: value,
            line: this.line,
            colmun: this.colmun
         }

         properties.push(property);

         if(this.at().type != Type.CloseBrace) {
            this.except(Type.Comma);
         }
      }
      this.except(Type.CloseBrace);

      let obj: Object = {
         type: "Obj",
         properties: properties,
         line: this.line,
         colmun: this.colmun
      }
      return obj;
   }



   protected parse_mathmatic_expr() : Expr {
      const main = this;
      function parse_additive_expr() : Expr {
         let left = parse_multiplactive_expr();
         
         while(main.at().value === "+" || main.at().value === "-") {
            let val: string = main.take().value;
            const right = parse_multiplactive_expr();
            left = {
               type: "BinaryExpr",
               left: left,
               ooperator: val,
               right: right,
               line: main.line,
               colmun: main.colmun
            } as BinaryExpr;
         }
         return left;
      }


      function parse_multiplactive_expr() : Expr {
         let left = main.parse_primary_expr();
         
         while(main.at().value === "*" || main.at().value === "/" || main.at().value === "%") {
            let val: string = main.take().value;
            const right = main.parse_primary_expr();
            left = {
               type: "BinaryExpr",
               left: left,
               ooperator: val,
               right: right,
               line: main.line,
               colmun: main.colmun
            } as BinaryExpr;
         }
         return left;
      }

      return parse_additive_expr();
   }


   protected parse_primary_expr() : Expr {
      switch(this.at().type) { 
         case Type.OpenParen:
            this.take();
            let expr = this.parse_expr();
            this.except(Type.CloseParen);
            return expr;
         case Type.id:
            return {
               type: "Id",
               symbol: this.take().value,
               line: this.line,
               colmun: this.colmun
            } as Id;
         case Type.str_type:
            return {
               type: "Str",
               value: this.take().value,
               line: this.line,
               colmun: this.colmun
            } as Str;
         case Type.num_type:
            return {
               type: "Num",
               value: +this.take().value,
               line: this.line,
               colmun: this.colmun
            } as Num;
         case Type.bool_type:
            return {
               type: "Bool",
               value: this.take().value == "true" ? true : false,
               line: this.line,
               colmun: this.colmun
            } as Bool;
         case Type.null_type:
            this.take();
            return {
               type: "Null",
               value: null,
               line: this.line,
               colmun: this.colmun
            } as Null;
         default:
            this.error("unexcepted ION", "AT1001");
            this.take();
            return {
               type: "Null",
               value: null,
               line: this.line,
               colmun: this.colmun
            } as Null;
      }
   }
}
