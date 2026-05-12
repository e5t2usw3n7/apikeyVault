```rust
            // ===== 过滤和排序密钥列表 =====

            // 先应用搜索关键词过滤和按环境过滤

            let filtered_keys: Vec<(usize, KeyEntry)> = self.key_list.iter().enumerate().filter(|(_, key)| {

                // 搜索匹配：名称、提供商、描述中包含搜索关键词（不区分大小写）

                let matches_search = self.key_search_query.is_empty()

                    || key.name.to_lowercase().contains(&self.key_search_query.to_lowercase())

                    || key.provider.to_lowercase().contains(&self.key_search_query.to_lowercase())

                    || key.description.as_ref().map_or(false, |d| d.to_lowercase().contains(&self.key_search_query.to_lowercase()));



                // 环境过滤匹配

                let matches_env = self.key_filter_env.is_empty()

                    || key.environment.to_string() == self.key_filter_env;



                matches_search && matches_env

            }).map(|(idx, key)| (idx, key.clone())).collect();



            // ===== 密钥表格渲染 =====

            if filtered_keys.is_empty() {

                // 没有数据时显示空状态提示

                ui.add_space(40.0);

                ui.vertical_centered(|ui| {

                    ui.label(RichText::new("🔑").size(48.0).color(theme.text_dim));

                    ui.add_space(8.0);

                    if self.key_list.is_empty() {

                        // Vault中没有任何密钥

                        ui.label(RichText::new("暂无密钥").size(16.0).color(theme.text_dim));

                        ui.add_space(8.0);

                        if ui.add(

                            egui::Button::new(RichText::new("➕ 添加第一个密钥").size(14.0).color(Color32::WHITE))

                                .fill(theme.accent)

                                .min_size(Vec2::new(160.0, 36.0))

                                .rounding(Rounding::same(6.0))

                        ).clicked() {

                            self.edit_form = KeyEditForm::default();

                            self.edit_is_new = true;

                            self.navigate_to(View::KeyEdit(None));

                        }

                    } else {

                        // 有密钥但搜索过滤没有匹配

                        ui.label(RichText::new("没有匹配的密钥").size(14.0).color(theme.text_dim));

                    }

                });

            } else {

                // ===== 表头渲染 =====

                let table_w = ui.available_width() - 55.0;

                // 6列宽度比例：名称20%, 提供商14%, 类型12%, 环境12%, 标签24%, 操作18%

                let tbl_col_widths = [

                    table_w * 0.20,

                    table_w * 0.14,

                    table_w * 0.12,

                    table_w * 0.12,

                    table_w * 0.24,

                    table_w * 0.18,

                ];

                // 表头背景（只有上方圆角）

                egui::Frame::none()

                    .fill(theme.bg_secondary)

                    .rounding(Rounding {

                        nw: 8.0,

                        ne: 8.0,

                        sw: 0.0,

                        se: 0.0,

                    })

                    .show(ui, |ui| {

                        ui.horizontal(|ui| {

                            ui.add_space(8.0);

                            let headers = ["名称", "提供商", "类型", "环境", "标签", "操作"];



                            for (i, header) in headers.iter().enumerate() {

                                // 表头项是可点击的排序按钮

                                let resp = ui.add_sized(

                                    Vec2::new(tbl_col_widths[i], 28.0),

                                    egui::Button::new(

                                        RichText::new(*header).size(12.0).strong().color(theme.text_secondary)

                                    ).fill(Color32::TRANSPARENT).frame(false),

                                );

                                if resp.clicked() {

                                    // 点击表头切换排序：点击同一列切换升降序，点击不同列切换为该列升序

                                    if self.key_sort_column == i {

                                        self.key_sort_ascending = !self.key_sort_ascending;

                                    } else {

                                        self.key_sort_column = i;

                                        self.key_sort_ascending = true;

                                    }

                                }

                            }

                        });

                    });



                // ===== 表格数据行（可滚动）=====

                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {

                    // 遍历过滤后的密钥列表

                    for (list_idx, (orig_idx, key)) in filtered_keys.iter().enumerate() {

                        // 交替行背景色（斑马纹）

                        let row_bg = if list_idx % 2 == 0 { theme.bg_card } else { theme.bg_secondary };



                        egui::Frame::none()

                            .fill(row_bg)

                            .show(ui, |ui| {

                                ui.horizontal(|ui| {

                                    ui.add_space(8.0);



                                    // ------ 名称列（蓝色链接样式，可点击进入详情）------

                                    if ui.add_sized(

                                        Vec2::new(tbl_col_widths[0], 32.0),

                                        egui::Button::new(

                                            RichText::new(&key.name).size(13.0).color(theme.accent)

                                        ).fill(Color32::TRANSPARENT).frame(false),

                                    ).clicked() {

                                        let idx = *orig_idx;

                                        self.decrypted_value = None;

                                        self.show_decrypted_value = false;

                                        self.selected_key_index = Some(idx);

                                        self.current_view = View::KeyDetail(idx);

                                    }



                                    // ------ 提供商列 ------

                                    ui.add_sized(

                                        Vec2::new(tbl_col_widths[1], 32.0),

                                        egui::Label::new(RichText::new(&key.provider).size(13.0).color(theme.text_primary)),

                                    );



                                    // ------ 类型列 ------

                                    ui.add_sized(

                                        Vec2::new(tbl_col_widths[2], 32.0),

                                        egui::Label::new(RichText::new(key.key_type.to_string()).size(13.0).color(theme.text_secondary)),

                                    );



                                    // ------ 环境列（带颜色标记）------

                                    let env_color = match key.environment.to_string().as_str() {

                                        "production" => theme.error,               // 生产环境=红色

                                        "staging" => theme.warning,                 // 预发布环境=黄色

                                        "development" => theme.success,             // 开发环境=绿色

                                        _ => theme.text_secondary,

                                    };

                                    ui.add_sized(

                                        Vec2::new(tbl_col_widths[3], 32.0),

                                        egui::Label::new(

                                            RichText::new(key.environment.to_string()).size(12.0).color(env_color).family(FontFamily::Monospace)

                                        ),

                                    );



                                    // ------ 标签列 ------

                                    let tags_str = if key.tags.is_empty() {

                                        "-".to_string()  // 无标签显示"-"

                                    } else {

                                        key.tags.join(", ")  // 多标签用逗号连接

                                    };

                                    ui.add_sized(

                                        Vec2::new(tbl_col_widths[4], 32.0),

                                        egui::Label::new(RichText::new(tags_str).size(12.0).color(theme.text_dim)),

                                    );



                                    // ------ 操作按钮列 ------

                                    ui.horizontal(|ui| {

                                        // 复制按钮

                                        if ui.add(

                                            egui::Button::new(RichText::new("📋").size(14.0))

                                                .fill(Color32::TRANSPARENT)

                                                .frame(false)

                                        ).on_hover_text("复制密钥值").clicked() {

                                            match self.vault.get_key(&key.name, &key.environment.to_string()) {

                                                Ok((_, value)) => self.copy_to_clipboard(&value),

                                                Err(e) => self.add_notification(Notification::error(format!("获取密钥失败: {}", e))),

                                            }

                                        }



                                        // 编辑按钮

                                        if ui.add(

                                            egui::Button::new(RichText::new("✏").size(14.0))

                                                .fill(Color32::TRANSPARENT)

                                                .frame(false)

                                        ).on_hover_text("编辑").clicked() {

                                            let idx = *orig_idx;

                                            self.edit_form = KeyEditForm::from_entry(key, &self.vault);

                                            self.edit_is_new = false;

                                            self.navigate_to(View::KeyEdit(Some(idx)));

                                        }



                                        // 删除按钮（需要确认）

                                        if ui.add(

                                            egui::Button::new(RichText::new("🗑").size(14.0))

                                                .fill(Color32::TRANSPARENT)

                                                .frame(false)

                                        ).on_hover_text("删除").clicked() {

                                            self.confirm_dialog = Some(ConfirmDialog {

                                                title: "删除密钥".to_string(),

                                                message: format!("确定要删除密钥 '{}' 吗？此操作不可恢复。", key.name),

                                                on_confirm_action: ConfirmAction::DeleteKey(

                                                    key.name.clone(),

                                                    key.environment.to_string(),

                                                ),

                                            });

                                        }

                                    });

                                });

                            });



                        // ===== 行间分隔线 =====

                        ui.painter().line_segment(

                            [

                                egui::pos2(ui.cursor().left() + 8.0, ui.cursor().top()),

                                egui::pos2(ui.cursor().right() - 8.0, ui.cursor().top()),

                            ],

                            Stroke::new(0.5, theme.border),  // 0.5px细线

                        );

                    }

                });

            }

        });

    }
```
这段代码里为什么表头位置合适，下面的表格数据行却没法呢和表头对齐呢