#![allow(dead_code)]
#![allow(unused_variables)]
use crate::index_comb::*;
use crate::ast::*;
use std::io::BufReader;
use std::io::prelude::*;
use regex::Regex;
use std::fs::File;
use std::io::Error;



#[derive(Debug,PartialEq, Eq, Clone)]
pub enum TableCell{
    CellInt(i64),
    CellString(String)
}

#[derive(Debug,PartialEq, Eq, Clone)]
pub struct TableData{
    pub header : Vec<String>,
    pub rows : Vec<Vec<Option<TableCell>>>
}



impl ColumnSelector {
    pub fn to_indexed(&self, tables : &Vec<TableData>, names : &Vec<String>) -> Result<IndexedColumnSelector, String>{
        let table_index : usize;
        let column_index : usize;
        match &self.table {
            Some(name) => {
                table_index = names.iter().position(|n| *n == *name)
                    .ok_or(format!("Specified table {} was not found for {}", name, self))?;
                
                column_index = tables[table_index].header.iter().position(|id| *id == self.field)
                    .ok_or(format!("Specified column {} was not found for in table {} for {}", self.field, name, self))?;

            },
            None => {
                let mut column_index_opt : Option<usize> = None;
                table_index = tables.iter().position( |table| {
                    column_index_opt = table.header.iter().position(|id| *id == self.field);
                    return matches!(column_index_opt, Some(x));
                }).ok_or(format!("Specified column {} was not found for in tables {:?} for {}", self.field, names, self))?;
                column_index = column_index_opt.ok_or("".to_string())?;
            }
        } 
        return Ok(IndexedColumnSelector{
            table : table_index,
            field : column_index
        });
    }
}

impl TableData {
    pub fn of_file(file : File) -> Result<Self, Error>{
        let mut header : Option<Vec<String>> = None;
        let mut rows : Vec<Vec<Option<TableCell>>> = Vec::new();
        let re = Regex::new("\".*\"").unwrap();
        let reader = BufReader :: new(file);
        for line_res in reader.lines(){
            let line = line_res.unwrap();
            if line.is_empty(){
                continue;
            }
            let cell_strings = line.split(',').map(|st|  { 
                let mut st_mut = st.to_string();
                st_mut.retain(|c| !c.is_whitespace() );
                return st_mut;
            
            }).collect();
            if header == None{
                header = Some(cell_strings);
            } else{
                let cells = cell_strings.iter().map(|cell|{
                    match re.captures_iter(cell).nth(0) {
                        None => {
                            match cell.parse::<i64>()  {
                                Ok(n) => Some(TableCell::CellInt(n)),
                                Err(_) => None
                            }
                        },
                        Some(_) => Some (TableCell::CellString(cell[1..cell.len() - 1].to_string()))
                    }
                }).collect();
                rows.push(cells);
            }
        }
        return Ok(TableData{
            header : header.unwrap(),
            rows : rows
        });
        
    }

    //can turn a lot of these vectors into slices if I care about efficiency (which I propaply don't)
    pub fn join_table(valid_row : fn(&Vec<Vec<Option<TableCell>>>) -> bool, tables : &Vec<TableData>) -> TableData{
        let n_tables = tables.len();
        let table_contents : Vec<_> = tables.iter().map(|table| &table.rows).collect();
        let table_headers : Vec<_>= tables.iter().map(|table| table.header.clone()).collect();
        let bounds : Vec<usize> = table_contents.iter().map(|table| table.len()).collect();
        let new_header : Vec<String> = table_headers.concat();
        let mut new_rows : Vec<Vec<Option<TableCell>>> = Vec::with_capacity(n_tables);
        for indices in given_bounds(bounds){
            let mut current_proposed_row : Vec<Vec<Option<TableCell>>> = Vec::with_capacity(n_tables);
            for (table_index, row_index) in indices.iter().enumerate(){
                current_proposed_row.push(table_contents[table_index][*row_index].clone());
            }
            if valid_row(&current_proposed_row){
                let new_row = current_proposed_row.concat();
                new_rows.push(new_row);
            }
        }
        return TableData{
            header : new_header,
            rows : new_rows
        };
    } 
}

#[cfg(test)]
mod tests {
    use crate::tables::*;
    use std::env::current_dir;
    use std::path::*;
    #[test]
    fn load_file_test1(){
        let test_header = vec!["name".to_string(), "age".to_string(), "id".to_string()];
        let row1 = vec![Some(TableCell::CellString("Lucas".to_string())), Some(TableCell::CellInt(24)), Some(TableCell::CellInt(0))];
        let row2 = vec![Some(TableCell::CellString("Harry".to_string())), Some(TableCell::CellInt(25)), Some(TableCell::CellInt(1))];
        let row3 = vec![Some(TableCell::CellString("".to_string())), None, Some(TableCell::CellInt(2))];
        let testtable = TableData {
            header: test_header,
            rows : vec![row1, row2, row3]
        };

        let mut curr_dir = current_dir().unwrap();
        curr_dir.push(Path::new("examples/test1.csv"));
        let filetable = TableData::of_file(File::open(curr_dir).unwrap()).unwrap();
        assert_eq!(testtable, filetable);
    }

    #[test]
    fn join_test(){
        let test_header1 = vec!["name".to_string(), "age".to_string(), "id".to_string()];
        let row11 = vec![Some(TableCell::CellString("Lucas".to_string())), Some(TableCell::CellInt(24)), Some(TableCell::CellInt(0))];
        let row12 = vec![Some(TableCell::CellString("Harry".to_string())), Some(TableCell::CellInt(25)), Some(TableCell::CellInt(1))];
        let row13 = vec![Some(TableCell::CellString("".to_string())), None, Some(TableCell::CellInt(2))];
        let testtable1 = TableData {
            header: test_header1.clone(),
            rows : vec![row11.clone(), row12.clone(), row13.clone()]
        };
        let test_header2 = vec!["name_".to_string(), "age_".to_string(), "id_".to_string()];
        let row21 = vec![Some(TableCell::CellString("Lucas_".to_string())), Some(TableCell::CellInt(24)), Some(TableCell::CellInt(0))];
        let row22 = vec![Some(TableCell::CellString("Harry_".to_string())), Some(TableCell::CellInt(25)), Some(TableCell::CellInt(1))];
        let row23 = vec![Some(TableCell::CellString("_".to_string())), None, Some(TableCell::CellInt(2))];
        let testtable2 = TableData {
            header: test_header2.clone(),
            rows : vec![row21.clone(), row22.clone(), row23.clone()]
        };

        let res_table1 = TableData::join_table( |vec| vec[0][1] == vec[1][1] , &(vec![testtable1.clone(), testtable2.clone()]));
        assert_eq!(res_table1.header, vec![test_header1.clone(), test_header2.clone()].concat());
        assert_eq!(res_table1.rows.len(), 3);
        //check rows
        assert_ne!(res_table1.rows.iter().find(| row | **row == vec![row11.clone(), row21.clone()].concat()), None );
        assert_ne!(res_table1.rows.iter().find(| row | **row == vec![row12.clone(), row22.clone()].concat()), None );
        assert_ne!(res_table1.rows.iter().find(| row | **row == vec![row13.clone(), row23.clone()].concat()), None );
        assert_eq!(res_table1.rows.iter().find(| row | **row == vec![row11.clone(), row22.clone()].concat()), None );
        let res_table2 = TableData::join_table( |vec| true , &(vec![testtable1.clone(), testtable2.clone()])); 
        assert_eq!(res_table2.header, vec![test_header1.clone(), test_header2.clone()].concat());
        assert_eq!(res_table2.rows.len(),9);
        assert_ne!(res_table2.rows.iter().find(| row | **row == vec![row11.clone(), row21.clone()].concat()), None );
        assert_ne!(res_table2.rows.iter().find(| row | **row == vec![row12.clone(), row21.clone()].concat()), None );
        assert_ne!(res_table2.rows.iter().find(| row | **row == vec![row13.clone(), row21.clone()].concat()), None );

        assert_ne!(res_table2.rows.iter().find(| row | **row == vec![row11.clone(), row22.clone()].concat()), None );
        assert_ne!(res_table2.rows.iter().find(| row | **row == vec![row12.clone(), row22.clone()].concat()), None );
        assert_ne!(res_table2.rows.iter().find(| row | **row == vec![row13.clone(), row22.clone()].concat()), None );

        assert_ne!(res_table2.rows.iter().find(| row | **row == vec![row11.clone(), row23.clone()].concat()), None );
        assert_ne!(res_table2.rows.iter().find(| row | **row == vec![row12.clone(), row23.clone()].concat()), None );
        assert_ne!(res_table2.rows.iter().find(| row | **row == vec![row13.clone(), row23.clone()].concat()), None );
        //the tables appear to be right
        let res_table3 = TableData::join_table( |vec| false , &(vec![testtable1.clone(), testtable2.clone()])); 
        assert_eq!(res_table3.header, vec![test_header1.clone(), test_header2.clone()].concat());
        assert_eq!(res_table3.rows.len(),0);
    }

}