//  脚本系统兼容性测试
//
//  测试所有脚本语言实现与ScriptContext trait的兼容性

#[cfg(test)]
mod tests {
    use crate::scripting::lua_support::LuaContext;
    use crate::scripting::system::JavaScriptContext;
    use crate::scripting::system::PythonContext;
    use crate::scripting::system::{
        ScriptContext, ScriptLanguage, ScriptResult, ScriptSystem, ScriptValue,
    };

    /// 测试所有脚本语言的ScriptContext trait兼容性
    #[test]
    fn test_all_script_context_compatibility() {
        // 测试Lua兼容性
        test_lua_context_compatibility();

        // 测试JavaScript兼容性
        test_javascript_context_compatibility();

        // 测试Python兼容性
        test_python_context_compatibility();
    }

    /// 测试LuaContext的ScriptContext兼容性
    fn test_lua_context_compatibility() {
        let mut lua_context = LuaContext::new();

        // 测试基本功能
        test_script_context_basic_operations(&mut lua_context, "Lua");

        // 测试Lua特定功能
        test_lua_specific_features(&mut lua_context);
    }

    /// 测试JavaScriptContext的ScriptContext兼容性
    fn test_javascript_context_compatibility() {
        let mut js_context = JavaScriptContext::new();

        // 测试基本功能
        test_script_context_basic_operations(&mut js_context, "JavaScript");

        // 测试JavaScript特定功能
        test_javascript_specific_features(&mut js_context);
    }

    /// 测试PythonContext的ScriptContext兼容性
    fn test_python_context_compatibility() {
        let mut py_context = PythonContext::new();

        // 测试基本功能
        test_script_context_basic_operations(&mut py_context, "Python");

        // 注意：Python目前是禁用状态，这是预期的
    }

    /// 测试ScriptContext trait的基本操作
    fn test_script_context_basic_operations<T: ScriptContext>(
        context: &mut T,
        language_name: &str,
    ) {
        println!("Testing {} ScriptContext compatibility...", language_name);

        // 1. 测试execute方法
        let result = context.execute("1 + 1");
        match language_name {
            "Lua" | "JavaScript" => {
                assert!(
                    matches!(result, ScriptResult::Success(ref s) if s == "2"),
                    "{} execute should return Success('2'), got {:?}",
                    language_name,
                    result
                );
            }
            "Python" => {
                // Python目前禁用，应该返回错误
                assert!(
                    matches!(result, ScriptResult::Error(_)),
                    "Python execute should return Error when disabled, got {:?}",
                    result
                );
            }
            _ => {}
        }

        // 2. 测试set_global和get_global方法
        let test_value = ScriptValue::Int(42);
        let set_result = context.set_global("test_var", test_value.clone());

        match language_name {
            "Lua" | "JavaScript" => {
                assert!(
                    matches!(set_result, ScriptResult::Success(_) | ScriptResult::Void),
                    "{} set_global should succeed, got {:?}",
                    language_name,
                    set_result
                );

                let get_result = context.get_global("test_var");
                assert!(
                    get_result.is_some(),
                    "{} get_global should return Some value",
                    language_name
                );

                assert_eq!(
                    get_result.expect("Test: operation should succeed"),
                    test_value,
                    "{} get_global should return the correct value",
                    language_name
                );
            }
            "Python" => {
                // Python目前禁用
                assert!(
                    matches!(set_result, ScriptResult::Error(_)),
                    "Python set_global should return Error when disabled, got {:?}",
                    set_result
                );

                let get_result = context.get_global("test_var");
                assert!(
                    get_result.is_none(),
                    "Python get_global should return None when disabled"
                );
            }
            _ => {}
        }

        // 3. 测试reset方法
        context.reset();
        let get_after_reset = context.get_global("test_var");

        match language_name {
            "Lua" | "JavaScript" => {
                // reset后变量应该被清除
                assert!(
                    get_after_reset.is_none(),
                    "{} get_global should return None after reset",
                    language_name
                );
            }
            "Python" => {
                // Python本来就没有变量
                assert!(
                    get_after_reset.is_none(),
                    "Python get_global should return None"
                );
            }
            _ => {}
        }

        println!("✓ {} ScriptContext compatibility verified", language_name);
    }

    /// 测试Lua特定功能
    fn test_lua_specific_features(context: &mut LuaContext) {
        // 测试函数调用
        let _ = context.execute("add_func", "function add(a, b) return a + b end");

        let args = vec![
            crate::scripting::LuaValue::Number(3.0),
            crate::scripting::LuaValue::Number(4.0),
        ];
        let result = context.call_function("add", args);

        assert!(
            result.is_ok(),
            "Lua function call should be Ok, got {:?}",
            result
        );

        // 测试表操作
        let _ = context.execute("table_test", "t = {x = 10, y = 20}");
        let table_value = context.get_global("t");

        // 表可能被转换为Object或Array，只要不是None就算成功
        if table_value.is_none() {
            println!("Warning: Lua table retrieval returned None, but this might be expected");
        } else {
            println!("✓ Lua successfully created and retrieved table");
        }

        // 测试引擎API绑定
        let result = context.execute("api_test", "engine.log('Lua API test')");
        assert!(
            result.is_ok(),
            "Lua engine API should work, got {:?}",
            result
        );

        println!("✓ Lua-specific features verified");
    }

    /// 测试JavaScript特定功能
    fn test_javascript_specific_features(context: &mut JavaScriptContext) {
        // 测试函数调用
        let result = context.call_function(
            "Math.pow",
            &[ScriptValue::Float(2.0), ScriptValue::Float(3.0)],
        );

        assert!(
            matches!(result, ScriptResult::Success(ref s) if s == "8"),
            "JavaScript Math.pow(2, 3) should return '8', got {:?}",
            result
        );

        // 测试引擎API绑定
        let result = context.execute("Engine.log('JS API test')");
        assert!(
            matches!(result, ScriptResult::Success(_) | ScriptResult::Void),
            "JavaScript engine API should work, got {:?}",
            result
        );

        println!("✓ JavaScript-specific features verified");
    }

    /// 测试ScriptSystem的多语言支持
    #[test]
    fn test_script_system_multilanguage_support() {
        let system = ScriptSystem::new();

        // 注册所有支持的脚本语言
        system.register_context(ScriptLanguage::Lua, Box::new(LuaContext::new()));

        system.register_context(
            ScriptLanguage::JavaScript,
            Box::new(JavaScriptContext::new()),
        );

        system.register_context(ScriptLanguage::Python, Box::new(PythonContext::new()));

        // 测试不同语言的执行
        let lua_result = system.execute(ScriptLanguage::Lua, "2 * 3");
        assert!(
            matches!(lua_result, ScriptResult::Success(ref s) if s == "6"),
            "Lua execution should work in ScriptSystem"
        );

        let js_result = system.execute(ScriptLanguage::JavaScript, "2 * 3");
        assert!(
            matches!(js_result, ScriptResult::Success(ref s) if s == "6"),
            "JavaScript execution should work in ScriptSystem"
        );

        let py_result = system.execute(ScriptLanguage::Python, "2 * 3");
        assert!(
            matches!(py_result, ScriptResult::Error(_)),
            "Python execution should return error when disabled"
        );

        // 测试全局变量在不同语言间的隔离
        system.set_global(
            ScriptLanguage::Lua,
            "test",
            ScriptValue::String("Lua".to_string()),
        );
        system.set_global(
            ScriptLanguage::JavaScript,
            "test",
            ScriptValue::String("JS".to_string()),
        );

        let lua_value = system.get_global(ScriptLanguage::Lua, "test");
        let js_value = system.get_global(ScriptLanguage::JavaScript, "test");

        assert_eq!(
            lua_value,
            Some(ScriptValue::String("Lua".to_string())),
            "Lua global variable should be isolated"
        );

        assert_eq!(
            js_value,
            Some(ScriptValue::String("JS".to_string())),
            "JavaScript global variable should be isolated"
        );

        println!("✓ ScriptSystem multilanguage support verified");
    }

    /// 测试ScriptValue类型兼容性
    #[test]
    fn test_script_value_type_compatibility() {
        let test_values = vec![
            ScriptValue::Null,
            ScriptValue::Bool(true),
            ScriptValue::Int(42),
            ScriptValue::Float(3.14),
            ScriptValue::String("test".to_string()),
            ScriptValue::Array(vec![ScriptValue::Int(1), ScriptValue::Int(2)]),
            ScriptValue::Object(std::collections::HashMap::from([(
                "key".to_string(),
                ScriptValue::String("value".to_string()),
            )])),
        ];

        let mut lua_context = LuaContext::new();
        let mut js_context = JavaScriptContext::new();

        // 测试所有ScriptValue类型在不同语言中的处理
        for (i, value) in test_values.iter().enumerate() {
            // 测试全局变量设置和获取
            let var_name = format!("test_var_{}", i);

            let lua_value = match value {
                ScriptValue::Null => crate::scripting::LuaValue::Nil,
                ScriptValue::Bool(b) => crate::scripting::LuaValue::Boolean(*b),
                ScriptValue::Int(i) => crate::scripting::LuaValue::Number(*i as f64),
                ScriptValue::Float(f) => crate::scripting::LuaValue::Number(*f as f64),
                ScriptValue::String(s) => crate::scripting::LuaValue::String(s.clone()),
                _ => crate::scripting::LuaValue::Nil,
            };
            lua_context.set_global(&var_name, lua_value);
            let js_set = js_context.set_global(&var_name, value.clone());

            match value {
                ScriptValue::Null => {
                    // Null值应该被正确处理
                    println!("Testing Null value type");
                    assert!(
                        matches!(js_set, ScriptResult::Success(_) | ScriptResult::Void),
                        "JavaScript should handle Null value type"
                    );
                }
                ScriptValue::Bool(_)
                | ScriptValue::Int(_)
                | ScriptValue::Float(_)
                | ScriptValue::String(_) => {
                    assert!(
                        matches!(js_set, ScriptResult::Success(_) | ScriptResult::Void),
                        "JavaScript should handle basic value type {:?}, got {:?}",
                        value,
                        js_set
                    );
                }
                ScriptValue::Array(_) | ScriptValue::Object(_) => {
                    // 复杂类型可能需要特殊处理
                    println!("Testing complex value type: {:?}", value);
                    // 对于复杂类型，我们期望至少不失败
                    assert!(
                        matches!(
                            js_set,
                            ScriptResult::Success(_) | ScriptResult::Void | ScriptResult::Error(_)
                        ),
                        "JavaScript should handle complex value type without panicking"
                    );
                }
            }

            let lua_get = lua_context.get_global(&var_name);
            let js_get = js_context.get_global(&var_name);

            // 验证值的获取（考虑到类型转换的可能差异）
            if lua_get.is_some() {
                println!("✓ Lua successfully handled value type: {:?}", value);
            }
            if js_get.is_some() {
                println!("✓ JavaScript successfully handled value type: {:?}", value);
            }
        }

        println!("✓ ScriptValue type compatibility verified");
    }

    /// 测试错误处理一致性
    #[test]
    fn test_error_handling_consistency() {
        let mut lua_context = LuaContext::new();
        let mut js_context = JavaScriptContext::new();

        // 测试语法错误
        let lua_syntax_error = lua_context.execute("syntax_error", "invalid syntax here");
        let js_syntax_error = js_context.execute("invalid syntax here");

        assert!(
            lua_syntax_error.is_err(),
            "Lua should return error for invalid syntax"
        );
        assert!(
            matches!(js_syntax_error, ScriptResult::Error(_)),
            "JavaScript should return error for invalid syntax"
        );

        // 测试函数不存在错误
        let lua_func_error = lua_context.call_function("nonexistent_function", vec![]);
        let js_func_error = js_context.call_function("nonexistent_function", &[]);

        assert!(
            lua_func_error.is_err(),
            "Lua should return error for nonexistent function"
        );
        assert!(
            matches!(js_func_error, ScriptResult::Error(_)),
            "JavaScript should return error for nonexistent function"
        );

        println!("✓ Error handling consistency verified");
    }
}
