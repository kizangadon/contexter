import { useState } from 'react';
import { useNavigate } from 'react-router';
import { ArrowUpRight } from 'lucide-react';
import { useSkills } from '@/api/hooks';
import { PageHeader } from '@/components/layout/PageHeader';
import { EmptyState } from '@/components/ui/EmptyState';
import { FilterBar } from '@/components/ui/FilterBar';
import { LoadingSkeleton } from '@/components/ui/LoadingSkeleton';
import { SkillCard } from './SkillCard';

export function SkillRegistryPage() {
  const [categoryFilter, setCategoryFilter] = useState('all');
  const navigate = useNavigate();

  const { data: allSkills, isLoading } = useSkills(undefined);

  // Client-side filter when a category is selected — avoids a second API call
  const skills = categoryFilter === 'all'
    ? allSkills
    : allSkills?.filter((s) => s.category === categoryFilter);

  const categories = Array.from(
    new Set(allSkills?.map((s) => s.category) ?? []),
  ).sort();

  const filterOptions = [
    { value: 'all', label: 'All Categories' },
    ...categories.map((cat) => ({ value: cat, label: cat })),
  ];

  return (
    <div>
      <PageHeader title="Skills" />

      {/* Filter bar */}
      <FilterBar
        className="mb-lg"
        filters={[
          {
            key: 'category',
            label: 'Category',
            options: filterOptions,
            value: categoryFilter,
            onChange: setCategoryFilter,
          },
        ]}
      />

      {/* Loading state */}
      {isLoading && (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {Array.from({ length: 6 }, (_, i) => (
            <LoadingSkeleton key={i} variant="card" />
          ))}
        </div>
      )}

      {/* Empty state */}
      {!isLoading && (skills ?? []).length === 0 && (
        <EmptyState
          icon={ArrowUpRight}
          title="No skills found"
          message={categoryFilter !== 'all'
            ? `No skills with the category "${categoryFilter}" were found.`
            : 'No skills are available yet. Skills will appear here once they are created.'}
        />
      )}

      {/* Skill card grid */}
      {!isLoading && (skills ?? []).length > 0 && (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {skills?.map((skill) => (
            <SkillCard
              key={skill.id}
              skill={skill}
              onClick={(id) => navigate(`/skills/${id}`)}
            />
          ))}
        </div>
      )}
    </div>
  );
}
