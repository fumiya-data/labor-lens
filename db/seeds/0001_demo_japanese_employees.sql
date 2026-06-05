-- LaborLens demo seed: 1000 fictional Japanese employee records.
-- This seed is for local portfolio/demo use only. It must not be mixed with
-- real personnel data.

CREATE SCHEMA IF NOT EXISTS laborlens;

CREATE TABLE IF NOT EXISTS laborlens.demo_employees (
    employee_id TEXT NOT NULL,
    display_name TEXT NOT NULL,
    department TEXT NOT NULL,
    store_name TEXT NOT NULL,
    employment_type TEXT NOT NULL,
    role_name TEXT NOT NULL,
    hired_on DATE NOT NULL,
    seed_version TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT pk_demo_employees PRIMARY KEY (employee_id),
    CONSTRAINT ck_demo_employees_employee_id CHECK (employee_id ~ '^EMP-[0-9]{4}$'),
    CONSTRAINT ck_demo_employees_seed_version CHECK (seed_version = 'demo_japanese_employees.v1')
);

TRUNCATE TABLE laborlens.demo_employees;

WITH
family_names(ordinality, name) AS (
    VALUES
        (1, '佐藤'), (2, '鈴木'), (3, '高橋'), (4, '田中'), (5, '伊藤'),
        (6, '渡辺'), (7, '山本'), (8, '中村'), (9, '小林'), (10, '加藤'),
        (11, '吉田'), (12, '山田'), (13, '佐々木'), (14, '山口'), (15, '松本'),
        (16, '井上'), (17, '木村'), (18, '林'), (19, '清水'), (20, '斎藤')
),
given_names(ordinality, name) AS (
    VALUES
        (1, '陽菜'), (2, '結衣'), (3, '葵'), (4, '凛'), (5, '美咲'),
        (6, '翔太'), (7, '蓮'), (8, '悠真'), (9, '大和'), (10, '湊'),
        (11, '直樹'), (12, '拓也'), (13, '真央'), (14, '彩'), (15, '優子'),
        (16, '健太'), (17, '誠'), (18, '舞'), (19, '亮'), (20, '恵')
),
departments(ordinality, name) AS (
    VALUES
        (1, '営業部'), (2, '販売部'), (3, '人事部'), (4, '経理部'), (5, '物流部'),
        (6, '製造部'), (7, '情報システム部'), (8, '商品管理部'), (9, 'カスタマー支援部'), (10, '企画部')
),
stores(ordinality, name) AS (
    VALUES
        (1, '東京東店'), (2, '東京西店'), (3, '横浜店'), (4, '千葉店'), (5, 'さいたま店'), (6, '名古屋店'),
        (7, '大阪北店'), (8, '大阪南店'), (9, '京都店'), (10, '神戸店'), (11, '福岡店'), (12, '札幌店')
),
employment_types(ordinality, name) AS (
    VALUES (1, '正社員'), (2, '契約社員'), (3, 'パート'), (4, 'アルバイト')
),
roles(ordinality, name) AS (
    VALUES (1, 'スタッフ'), (2, '主任'), (3, '店長'), (4, '事務担当'), (5, '部門責任者')
),
numbered AS (
    SELECT
        gs AS employee_number,
        ((gs - 1) % 20) + 1 AS family_index,
        (((gs - 1) / 20) % 20) + 1 AS given_index,
        ((gs - 1) % 10) + 1 AS department_index,
        ((gs - 1) % 12) + 1 AS store_index,
        ((gs - 1) % 4) + 1 AS employment_type_index,
        ((gs - 1) % 5) + 1 AS role_index
    FROM generate_series(1, 1000) AS gs
)
INSERT INTO laborlens.demo_employees (
    employee_id,
    display_name,
    department,
    store_name,
    employment_type,
    role_name,
    hired_on,
    seed_version
)
SELECT
    format('EMP-%s', lpad(employee_number::text, 4, '0')),
    family_names.name || ' ' || given_names.name,
    departments.name,
    stores.name,
    employment_types.name,
    roles.name,
    make_date(2014 + ((employee_number - 1) % 11), 1 + ((employee_number - 1) % 12), 1),
    'demo_japanese_employees.v1'
FROM numbered
JOIN family_names ON family_names.ordinality = numbered.family_index
JOIN given_names ON given_names.ordinality = numbered.given_index
JOIN departments ON departments.ordinality = numbered.department_index
JOIN stores ON stores.ordinality = numbered.store_index
JOIN employment_types ON employment_types.ordinality = numbered.employment_type_index
JOIN roles ON roles.ordinality = numbered.role_index;

DO $$
DECLARE
    seeded_count INTEGER;
BEGIN
    SELECT count(*) INTO seeded_count
    FROM laborlens.demo_employees
    WHERE seed_version = 'demo_japanese_employees.v1';

    IF seeded_count <> 1000 THEN
        RAISE EXCEPTION 'expected 1000 demo employees, got %', seeded_count;
    END IF;
END $$;

COMMENT ON TABLE laborlens.demo_employees IS 'Fictional Japanese employee demo seed for local UI use-case sample loading.';
